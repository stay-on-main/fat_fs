#![no_std]

pub mod storage;
pub mod table;
pub mod storage_io;
pub mod stream;
pub mod dir;
pub mod dir_entry;
pub mod file;
pub mod path;

use path::Path;
use file::File;
use dir::Dir;
use dir_entry::DirEntry;
use storage::{Storage, StorageRead, StorageWrite};
use storage_io::StorageIo;
use table::{FatTable, FatType};
use core::cell::RefCell;
use stream::Stream;

fn u32_from_bytes(bytes: &[u8]) -> u32 {
    u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) |
    (u32::from(bytes[2]) << 16) | (u32::from(bytes[3]) << 24)
}

fn u16_from_bytes(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

pub struct Fs <T: StorageIo> {
    pub storage: RefCell<Storage<T>>,
    pub table: FatTable,

    sector_size: u32,
    sectors_in_cluster: u32,
    data_area_first_sector: u32,
    root_directory_first_sector: u32,
    root_cluster: u32,
}

impl <T: StorageIo> Fs <T> {
    pub fn new(storage_io: T) -> Result<Self, bool> {
        let mut storage = Storage::new(storage_io);
        let mut bpb = [0u8;512];
        storage.read(0, 0, &mut bpb)?;
        
        let sector_size = u32::from(u16_from_bytes(&bpb[11..]));
        let sectors_in_cluster = u32::from(bpb[13]);
        let reserved_sectors_count = u32::from(u16_from_bytes(&bpb[14..]));
        let num_fats = u32::from(bpb[16]);
        let root_entity_count = u32::from(u16_from_bytes(&bpb[17..])); // fat32 : 0
        let total_sectors_16 = u32::from(u16_from_bytes(&bpb[19..])); // fat32: 0
        let fat_size_16 = u32::from(u16_from_bytes(&bpb[22..])); // fat32: 0

        let fat_size = if fat_size_16 != 0 {
            fat_size_16
        } else {
            // fat_size_32
            u32_from_bytes(&bpb[36..])
        };

        let total_sectors = if total_sectors_16 != 0 {
            total_sectors_16
        } else {
            // total_sectors_32
            u32_from_bytes(&bpb[32..])
        };
        // 1. Determine the count of sectors occupied by the root directory
        let root_dir_sectors = ((root_entity_count * 32) + (sector_size - 1)) / sector_size;
        // 2. Determine the count of sectors in the data region of the volume
        let data_sec = total_sectors - (reserved_sectors_count + (num_fats * fat_size) + root_dir_sectors); 

        let count_of_clusters = data_sec / sectors_in_cluster;

        let fat_type = if count_of_clusters < 4085 { 
            //println!("FAT12");
            // fat 12 not supported
            #[cfg(feature = "fat12_disable")]
            return Err(false); 
            FatType::Fat12
        } else if count_of_clusters < 65525 {
            //println!("FAT16");
            #[cfg(feature = "fat16_disable")]
            return Err(false); 
            FatType::Fat16
        } else {     
            //println!("FAT32");
            #[cfg(feature = "fat32_disable")]
            return Err(false); 
            FatType::Fat32
        };
        /*
        println!("sector_size: {}", sector_size);
        println!("sectors_in_cluster: {}", sectors_in_cluster);
        println!("root_dir_sectors: {}", root_dir_sectors);
        println!("reserved_sectors_count: {}", reserved_sectors_count);
        println!("num_fats: {}", num_fats);
        println!("fat_size: {}", fat_size);
        */
        let root_directory_first_sector = reserved_sectors_count + (num_fats * fat_size);
        let data_area_first_sector = root_directory_first_sector + root_dir_sectors;
        /*
        println!("root_directory_first_sector {}", root_directory_first_sector);
        println!("data_area_first_sector {}", data_area_first_sector);
        */

        let root_cluster = match fat_type {
            FatType::Fat32 => u32_from_bytes(&bpb[44..]),
            _ => 2,
        };
        
        Ok(Fs {
            storage: RefCell::new(storage),
            table: FatTable::new(fat_type, reserved_sectors_count, fat_size, sector_size),
            sector_size,
            sectors_in_cluster,
            data_area_first_sector,
            root_directory_first_sector,
            root_cluster,
        })
    }

    pub fn root_dir(&self) -> Dir<T> {
        match self.table.fat_type {
            FatType::Fat32 => Dir::new(self, self.root_cluster),
            _ => Dir::new(self, 0),
        }
    }

    pub fn root_dir_cluster(&self) -> u32 {
        match self.table.fat_type {
            FatType::Fat32 => self.root_cluster,
            _ => 0,
        }
    }

    pub fn cluster_to_sector(&self, cluster: u32) -> u32 {
        if cluster != 0 {
            self.data_area_first_sector + (cluster - self.root_cluster) * self.sectors_in_cluster
        } else {
            // If the parent directory is the root directory 
            // (which is statically allocated and doesn't have a cluster number),
            // cluster number 0x0000 is specified here.
            self.root_directory_first_sector
        }
    }
}
