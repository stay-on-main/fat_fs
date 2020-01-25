use super::StorageRead;
use super::StorageWrite;

pub enum FatType {
#[cfg(not(feature = "fat32_disable"))]
    Fat32,
#[cfg(not(feature = "fat16_disable"))]
    Fat16,
#[cfg(not(feature = "fat12_disable"))]
    Fat12
}

pub enum FatValue {
    Next(u32),
    Last,
    Free,
    Bad,
}

pub struct FatTable {
    pub fat_type: FatType,
    first_block: u32,
    block_count: u32,
    block_size: u32,
}

fn u32_from_bytes(bytes: &[u8]) -> u32 {
    u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) |
    (u32::from(bytes[2]) << 16) | (u32::from(bytes[3]) << 24)
}

fn u16_from_bytes(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

impl FatTable {
    pub fn new(fat_type: FatType, first_block: u32, block_count: u32, block_size: u32) -> Self {
        FatTable { fat_type, first_block, block_count, block_size }
    }

    #[cfg(not(feature = "fat32_disable"))]
    fn fat_32_get<T: StorageRead>(&self, io: &mut T, cluster: u32) -> Result<FatValue, bool> {
        let block = (cluster * 4) / self.block_size;
        let offset = (cluster * 4) % self.block_size;

        if block >= self.block_count {
            return Err(false);
        }

        let mut buf = [0u8; 4];
        io.read(self.first_block + block, offset as usize, &mut buf)?;
        let val = u32_from_bytes(&buf) & 0x0FFF_FFFF;

        match val {
            0 => Ok(FatValue::Free),
            0x0FFF_FFF7 => Ok(FatValue::Bad),
            0x0FFF_FFF8..=core::u32::MAX => Ok(FatValue::Last),
            value => Ok(FatValue::Next(value)),
        }
    }

    #[cfg(not(feature = "fat32_disable"))]
    #[cfg(not(feature = "fs_read_only"))]
    fn fat_32_set<T: StorageWrite>(&self, io: &mut T, cluster: u32, value: FatValue) -> Result<(), bool> {
        let raw_value = match value {
            FatValue::Next(n) => n & 0x0FFF_FFFF,
            FatValue::Last => 0x0FFF_FFF8,
            FatValue::Free => 0,
            FatValue::Bad => 0x0FFF_FFF7,
        };

        let block = (cluster * 4) / self.block_size;
        let offset = (cluster * 4) % self.block_size;
        io.write(self.first_block + block, offset as usize, &raw_value.to_le_bytes())
    }

    #[cfg(not(feature = "fat16_disable"))]
    fn fat_16_get<T: StorageRead>(&self, io: &mut T, cluster: u32) -> Result<FatValue, bool> {
        let block = (cluster * 2) / self.block_size;
        let offset = (cluster * 2) % self.block_size;

        if block >= self.block_count {
            return Err(false);
        }

        let mut buf = [0u8; 2];
        io.read(self.first_block + block, offset as usize, &mut buf)?;
        let val = u16_from_bytes(&buf);

        match val {
            0 => Ok(FatValue::Free),
            0xFFF7 => Ok(FatValue::Bad),
            0xFFF8..=0xFFFF => Ok(FatValue::Last),
            value => Ok(FatValue::Next(value as u32)),
        }
    }

    #[cfg(not(feature = "fat16_disable"))]
    #[cfg(not(feature = "fs_read_only"))]
    fn fat_16_set<T: StorageWrite>(&self, io: &mut T, cluster: u32, value: FatValue) -> Result<(), bool> {
        let raw_value = match value {
            FatValue::Next(n) => n & 0xFFFF,
            FatValue::Last => 0xFFF8,
            FatValue::Free => 0,
            FatValue::Bad => 0xFFF7,
        };

        let block = (cluster * 2) / self.block_size;
        let offset = (cluster * 2) % self.block_size;
        let raw_value = raw_value as u16;
        io.write(self.first_block + block, offset as usize, &raw_value.to_le_bytes())
    }

    #[cfg(not(feature = "fat12_disable"))]
    fn fat_12_get<T: StorageRead>(&self, io: &mut T, cluster: u32) -> Result<FatValue, bool> {
        let block = (cluster + (cluster / 2)) / self.block_size;
        let offset = (cluster + (cluster / 2)) % self.block_size;

        if block >= self.block_count {
            return Err(false);
        }

        let mut buf = [0u8; 2];
        io.read(self.first_block + block, offset as usize, &mut buf)?;
        let val = u16_from_bytes(&buf);

        let raw_value = if cluster & 1 == 0 {
            (val & 0x0FFF) as u32
        } else {
            (val >> 4) as u32
        };

        match raw_value {
            0 => Ok(FatValue::Free),
            0xFF7 => Ok(FatValue::Bad),
            0xFF8..=0xFFF => Ok(FatValue::Last),
            value => Ok(FatValue::Next(value)),
        }
    }

    #[cfg(not(feature = "fat12_disable"))]
    #[cfg(not(feature = "fs_read_only"))]
    fn fat_12_set<T: StorageRead + StorageWrite>(&self, io: &mut T, cluster: u32, value: FatValue) -> Result<(), bool> {
        let raw_value = match value {
            FatValue::Next(n) => n & 0xFFF,
            FatValue::Last => 0xFF8,
            FatValue::Free => 0,
            FatValue::Bad => 0xFF7,
        };

        let block = (cluster + (cluster / 2)) / self.block_size;
        let offset = (cluster + (cluster / 2)) % self.block_size;
        let mut buf = [0u8; 2];
        io.read(self.first_block + block, offset as usize, &mut buf)?;

        if cluster & 1 == 0 {
            buf[0] = raw_value as u8;
            buf[1] = (buf[1] & 0x0f) | (((raw_value >> 8) & 0x0f) as u8);
        } else {
            buf[0] = (buf[0] & 0xf0) | (((raw_value & 0x0f) << 4) as u8);
            buf[1] = (raw_value >> 8) as u8;
        }
        
        io.write(self.first_block + block, offset as usize, &buf)
    }

    pub fn get<T: StorageRead>(&self, io: &mut T, cluster: u32) -> Result<FatValue, bool> {
        match &self.fat_type {
            #[cfg(not(feature = "fat32_disable"))]
            FatType::Fat32 => self.fat_32_get(io, cluster),
            #[cfg(not(feature = "fat16_disable"))]
            FatType::Fat16 => self.fat_16_get(io, cluster),
            #[cfg(not(feature = "fat12_disable"))]
            FatType::Fat12 => self.fat_12_get(io, cluster),
        }
    }

    #[cfg(not(feature = "fs_read_only"))]
    pub fn set<T: StorageRead + StorageWrite>(&self, io: &mut T, cluster: u32, value: FatValue) -> Result<(), bool> {
        match &self.fat_type {
            #[cfg(not(feature = "fat32_disable"))]
            FatType::Fat32 => self.fat_32_set(io, cluster, value),
            #[cfg(not(feature = "fat16_disable"))]
            FatType::Fat16 => self.fat_16_set(io, cluster, value),
            #[cfg(not(feature = "fat12_disable"))]
            FatType::Fat12 => self.fat_12_set(io, cluster, value),
        }
    }
}
