use std::fs::File;
use std::io::{ Read, Seek, SeekFrom, Write };
use core::cell::{RefCell};
pub use super::fat_fs::storage_io::StorageIo;
use core::ops::DerefMut;

const IMAGE_SECTOR_SIZE: usize = 512;

pub struct Image {
    file: RefCell<File>,
    read_count: RefCell<u32>,
}

impl StorageIo for Image {
    fn block_size(&self) -> u32 {
        IMAGE_SECTOR_SIZE as u32
    }

    fn write(&self, block: u32, data: &[u8]) -> Result<(), bool> {
        assert!(data.len() == IMAGE_SECTOR_SIZE);
        println!("write sector: {}", block);
        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start((block as u64) * (IMAGE_SECTOR_SIZE as u64))).unwrap();
        if let Ok(_) = file.write(data) {
            Ok(())
        } else {
            Err(false)
        }
    }

    fn read(&self, block: u32, data: &mut [u8]) -> Result<(), bool> {
        assert!(data.len() == IMAGE_SECTOR_SIZE);
        
        let mut read_count = self.read_count.borrow_mut();
        let read_count_clean = read_count.deref_mut();
        *read_count_clean += 1;

        let mut file = self.file.borrow_mut();
        file.seek(SeekFrom::Start((block as u64) * (IMAGE_SECTOR_SIZE as u64))).unwrap();

        if let Ok(_) = file.read(data) {
            Ok(())
        } else {
            Err(false)
        }
    }

    fn block_count(&self) -> u32 {
        let mut file = self.file.borrow_mut();
        let len = file.seek(SeekFrom::End(0)).unwrap();
        (len / (IMAGE_SECTOR_SIZE as u64)) as u32
    }
}

pub fn new(path: &str) -> Image {
    println!("open image: {}", path);
    //let mut file = ;
    //let mut data = Vec::new();
    //file.read_to_end(&mut data).unwrap();
    Image { 
        file: RefCell::new(File::open(path).unwrap()),
        read_count: RefCell::new(0) 
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let mut read_count = self.read_count.borrow_mut();
        let read_count_clean = read_count.deref_mut();
        println!("Sector read count: {}", read_count_clean);
    }
}