use super::StorageIo;

const BLOCK_MAX_SIZE: usize = 4096;
const BLOCK_MIN_SIZE: usize = 512;

pub struct Storage<T: StorageIo> {
    io: T,
    cached_block: u32,
    data: [u8; BLOCK_MAX_SIZE],
    block_size: usize,
    block_count: u32,
    dirty: bool,
}

pub trait StorageRead {
    fn read(&mut self, block: u32, offset: usize, buf: &mut [u8]) -> Result<(), bool>;
}

pub trait StorageWrite {
    fn write(&mut self, block: u32, offset: usize, buf: &[u8]) -> Result<(), bool>;
    fn flush(&mut self) -> Result<(), bool>;
}

impl <T: StorageIo> Storage <T> {
    pub fn new(io: T) -> Self {
        let block_size = io.block_size() as usize;
        let block_count = io.block_count();

        assert!((block_size as usize) >= BLOCK_MIN_SIZE);
        assert!((block_size as usize) <= BLOCK_MAX_SIZE);
        assert!(((block_size as usize) % 512) == 0);
        
        Storage {
            io,
            cached_block: core::u32::MAX,
            data: [0u8; BLOCK_MAX_SIZE],
            block_size,
            block_count,
            dirty: false,
        }
    }

    fn sync(&mut self, block: u32) -> Result<(), bool> {
        if block >= self.block_count {
            return Err(false);
        }

        if block != self.cached_block {
            self.flush()?;
            self.io.read(block, &mut self.data[..self.block_size])?;
            self.cached_block = block;
        }
        Ok(())
    }
}

impl <T: StorageIo> StorageRead for Storage <T> {
    fn read(&mut self, block: u32, offset: usize, buf: &mut [u8]) -> Result<(), bool> {
        assert!(buf.len() <= self.block_size - offset);
        //println!("read: 0x{:x}", block * 512 + offset as u32);
        self.sync(block)?;
        let offset_end = offset + buf.len();
        buf[..].copy_from_slice(&self.data[offset..offset_end]);
        Ok(())
    }
}

impl <T: StorageIo> StorageWrite for Storage <T> {
    fn write(&mut self, block: u32, offset: usize, buf: &[u8]) -> Result<(), bool> {
        assert!(buf.len() <= self.block_size - offset);

        self.sync(block)?;
        self.dirty = true;
        let offset_end = offset + buf.len();
        self.data[offset..offset_end].copy_from_slice(&buf[..]);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), bool> {
        if self.dirty {
            self.io.write(self.cached_block, &self.data[..self.block_size])?;
            self.dirty = false;
        }
        Ok(())
    }
}

