pub trait StorageIo {
    fn block_size(&self) -> u32;
    fn block_count(&self) -> u32;
    fn read(&self, block: u32, data: &mut [u8]) -> Result<(), bool>;
    #[cfg(not(feature = "fs_read_only"))]
    fn write(&self, block: u32, data: &[u8]) -> Result<(), bool>;
}