use super::Fs;
use super::storage_io::StorageIo;
use crate::fs::storage::StorageRead;
use crate::fs::storage::StorageWrite;
use super::table::FatValue;
use core::ops::DerefMut;

pub enum StreamPos {
    Start(u32),
    Current(i32),
    End(i32),
}

pub struct Stream<'a, T: StorageIo> {
    pub fs: &'a Fs<T>,
    first_cluster: u32,
    current_cluster: u32,
    sector_in_cluster: u32,
    offset_in_sector: usize,
    global_offset: u32,
    lenear: bool,
}

impl <'a, T: StorageIo> Stream<'a, T> {
    pub fn new(fs: &Fs<T>, cluster: u32, lenear: bool) -> Stream<T> {
        Stream {
            fs,
            first_cluster: cluster,
            current_cluster: cluster,
            sector_in_cluster: 0,
            offset_in_sector: 0,
            global_offset: 0,
            lenear,
        }
    }

    fn sync(&mut self) -> Result<(), bool> {
        /*
        println!("offset_in_sector: {}", self.offset_in_sector);
        println!("self.fs.sectors_in_cluster: {}", self.fs.sectors_in_cluster);
        println!("self.sector_in_cluster : {}", self.sector_in_cluster );
        */
        if self.offset_in_sector as u32 >= self.fs.sector_size {
            if !self.lenear {
                if self.sector_in_cluster + 1 >= self.fs.sectors_in_cluster {
                    let mut storage = self.fs.storage.borrow_mut();
                    let storage_mut = storage.deref_mut();

                    match self.fs.table.get(storage_mut, self.current_cluster)? {
                        FatValue::Next(next) => self.current_cluster = next,
                        FatValue::Last => return Err(false),
                        FatValue::Bad | FatValue::Free => return Err(false),
                    }

                    //println!("next cluster: {}", self.current_cluster);
                    self.sector_in_cluster = 0;
                } else {
                    self.sector_in_cluster += 1;
                }
            } else {
                self.sector_in_cluster += 1;
            }
            self.offset_in_sector = 0;
        }
        Ok(())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, bool> {
        self.sync()?;
        let len = core::cmp::min(buf.len(), (self.fs.sector_size as usize) - self.offset_in_sector);
        let sector = self.fs.cluster_to_sector(self.current_cluster) + self.sector_in_cluster;
        self.fs.storage.borrow_mut().read(sector, self.offset_in_sector, &mut buf[..len])?;
        self.offset_in_sector += len;
        self.global_offset += len as u32;
        Ok(len)
    }
    
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, bool> {
        self.sync()?;
        let len = core::cmp::min(buf.len(), (self.fs.sector_size as usize) - self.offset_in_sector);
        let sector = self.fs.cluster_to_sector(self.current_cluster) + self.sector_in_cluster;
        self.fs.storage.borrow_mut().write(sector, self.offset_in_sector, &buf[..len])?;
        self.offset_in_sector += len;
        self.global_offset += len as u32;
        Ok(len)
    }
    
    fn get_cluster(&mut self, cluster: u32, skip: u32) -> Result<FatValue, bool> {
        let mut storage = self.fs.storage.borrow_mut();
        let storage_mut = storage.deref_mut();

        let mut cluster = cluster;

        for _ in 0..(skip as usize) {
            match self.fs.table.get(storage_mut, cluster)? {
                FatValue::Next(c) => cluster = c,
                FatValue::Bad => return Ok(FatValue::Bad),
                FatValue::Free => return Ok(FatValue::Free),
                FatValue::Last => return Ok(FatValue::Last),
            }
        }

        Ok(FatValue::Next(cluster))
    }

    pub fn seek(&mut self, pos: StreamPos) -> Result<u32, bool> {
        let new_pos = match pos {
            StreamPos::Current(c) => {
                if c == 0 {
                    return Ok(self.global_offset);
                }

                (self.global_offset as i32) + c
            },
            StreamPos::Start(s) => s as i32,
            StreamPos::End(_) => todo!(),
        };

        if new_pos < 0 {
            return Err(false);
        }

        let new_pos = new_pos as u32;
        let cluster_size = self.fs.sectors_in_cluster * self.fs.sector_size;
        
        if new_pos / cluster_size != self.global_offset / cluster_size {
            let (origin, skip) = if new_pos < self.global_offset {
                // start search from file origin
                let clusters_to_skip = new_pos % cluster_size;
                (self.first_cluster, clusters_to_skip)
            } else {
                // start search from current position in file
                let clusters_to_skip = new_pos % cluster_size;
                let origin = self.global_offset % cluster_size;
                let clusters_to_skip = clusters_to_skip - origin;
                (self.first_cluster, clusters_to_skip)
            };

            match self.get_cluster(origin, skip)? {
                FatValue::Next(n) => self.current_cluster = n,
                _ => return Err(false)
            }
        }

        self.sector_in_cluster = (new_pos % cluster_size) / self.fs.sector_size;
        self.offset_in_sector = ((new_pos % cluster_size) % self.fs.sector_size) as usize;

        self.global_offset = new_pos as u32;
        Ok(self.global_offset)
    }
}

