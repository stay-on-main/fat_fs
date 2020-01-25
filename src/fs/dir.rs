use super::stream::Stream;
use super::storage_io::StorageIo;
use super::Fs;
use super::dir_entry::DirEntry;

const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_FILE_NAME: u8 = 0x0f;

pub struct Dir <'a, T: StorageIo> {
    stream: Stream<'a, T>,
}

impl <'a, T: StorageIo> Dir<'a, T> {
    pub fn new(fs: &'a Fs<T>, cluster: u32) -> Self {
        Dir {
            stream: Stream::new(fs, cluster, cluster == 0),
        }
    }
}

fn byte_to_lowercase(byte: u8) -> u8 {
    if (byte >= b'A') && (byte <= b'Z') {
        byte - b'A' + b'a'
    } else {
        byte
    }
}

impl <'a, T: StorageIo> Iterator for Dir<'a, T> {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut data = [0u8; 32];
        
        let mut lfn_buf = [0u8; 256];
        let mut lfn_checksum = 0u8;
        let mut lfn_num = 0u8;
        let mut lfn_len = 0;

        while let Ok(_) = self.stream.read(&mut data) {
            let attr = data[11];

            if data[0] == NO_MORE_DIR_ENTRY {
                //println!("No more dir entry");
                break;
            }
    
            if data[0] == DELETED_DIR_ENTRY {
                //println!("Deleted entry");
                lfn_num = 0;
                continue;
            }
    
            if  (attr & ATTR_LONG_NAME_MASK) == ATTR_LONG_NAME {
                //Found an active long name sub-component
               // println!("Lfn part entry");
                let last_lfn = (data[0] & LAST_LONG_ENTRY_MASK) == LAST_LONG_ENTRY;
    
                if last_lfn {
                    lfn_checksum = data[LDIR_CHKSUM];
                } else if lfn_checksum != data[LDIR_CHKSUM] || (data[0] + 1 ) != lfn_num {
                    println!("Lfn corrupted part");
                    lfn_num = 0;
                    continue;
                }
                
                lfn_num = data[0] & (!LAST_LONG_ENTRY_MASK);

                if last_lfn {
                    lfn_len = (lfn_num as usize) * LFN_OFFSETS.len();
                }

                let lfn_offset = (lfn_num as usize - 1) * LFN_OFFSETS.len();
    
                for (i, &offset) in LFN_OFFSETS.iter().enumerate() {
                    if data[offset] != 0 {
                        lfn_buf[lfn_offset + i] = data[offset];
                    } else {
                        if last_lfn {
                            lfn_len = lfn_offset + i;
                        } else {
                            println!("Lfn corrupted part");
                            lfn_num = 0;
                        }
                        break;
                    }
                }
            } else {
                if (lfn_len != 0) && (lfn_num == 1) && (checksum(&data[0..11]) == lfn_checksum) {
                    //println!("Good lfn");
                    //print_str(&lfn_buf[..lfn_len]);
                } else {
                    //print!("Bad lfn");
                    /*
                    for &b in data.iter() {
                        print!("{:2x} ", b);
                    }
                    println!();
                    */
                    lfn_len = 0;

                    if (data[12] & 0x18) != 0 {
                        // Windows NT uses this byte in special cases to indicate that
                        // a portion of the filename should be displayed in lowercase.
                        
                            for i in 0..8 {
                                if data[i] == b' ' {
                                    break;
                                }

                                lfn_buf[lfn_len] = if (data[12] & 0x08) != 0 {
                                    byte_to_lowercase(data[i])
                                } else {
                                    data[i]
                                };

                                lfn_len += 1;
                            }

                        if (data[11] & (ATTR_VOLUME_ID | ATTR_DIRECTORY)) == 0 && (data[8] != b' ') {
                            // is file
                            lfn_buf[lfn_len] = b'.';
                            lfn_len += 1;
            
                            for i in 8..11 {
                                if data[i] == b' ' {
                                    break;
                                }
            
                                lfn_buf[lfn_len] = if (data[12] & 0x10) != 0 {
                                    byte_to_lowercase(data[i])
                                } else {
                                    data[i]
                                };

                                lfn_len +=1;
                            }
                        }
                    }
                }
                
                //print!("sfn ");
                //print_str(&data[..11]);

                let mut sfn_buf = [0u8; 12];
                let mut sfn_len = 0;
    
                for &b in data.iter().take(8) {
                    if b == b' ' {
                        break;
                    }
    
                    sfn_buf[sfn_len] = b;
                    sfn_len += 1;
                }
    
                if (data[11] & (ATTR_VOLUME_ID | ATTR_DIRECTORY)) == 0 && (data[8] != b' ') {
                    // is file
                    sfn_buf[sfn_len] = b'.';
                    sfn_len += 1;
    
                    for &b in data.iter().take(11).skip(8) {
                        if b == b' ' {
                            break;
                        }
    
                        sfn_buf[sfn_len] = b;
                        sfn_len += 1;
                    }
                }
    
                let cluster_hi = u32::from(super::u16_from_bytes(&data[20..]));
                let cluster_lo = u32::from(super::u16_from_bytes(&data[26..]));
                let cluster = (cluster_hi << 16) | cluster_lo;
                let size = super::u32_from_bytes(&data[28..]);

                return Some(DirEntry {
                    cluster,
                    size,
                    sfn_buf,
                    sfn_len,
                    lfn_buf,
                    lfn_len,
                    attr, 
                });
            }
        }
        //println!("read stream false");
        None
    }
}

const ATTR_LONG_NAME_MASK: u8 = 0x3f;
const ATTR_LONG_NAME: u8 = 0x0f;

const LAST_LONG_ENTRY: u8 = 0x40;
const LAST_LONG_ENTRY_MASK: u8 = 0x40 | 0x80;

const LDIR_CHKSUM: usize = 13;

const DELETED_DIR_ENTRY: u8 = 0xE5;
const NO_MORE_DIR_ENTRY: u8 = 0x00;

const DIR_ENTRY_SIZE: usize = 32;

const LFN_OFFSETS: [usize; 13] = [1, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30];

fn checksum(buf: &[u8]) -> u8 {
    let mut res = 0u8;

    for &b in buf.iter() {
        let tmp = ((res as u32) << 7) + ((res as u32) >> 1) + b as u32;
        res = tmp as u8;
    }
    res
}

fn print_str(s: &[u8]) {
    for c in s {
        print!("{}", *c as char);
    }

    println!();
}