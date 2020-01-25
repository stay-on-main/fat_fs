use super::stream::{Stream, StreamPos};
use super::storage_io::StorageIo;

pub struct File<'a, T: StorageIo> {
    stream: Stream<'a, T>,
    size: u32,
}

impl <'a, T: StorageIo> File<'a, T> {
    pub fn new(stream: Stream<'a, T>, size: u32) -> Self {
        File {
            stream,
            size,
        }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, bool> {
        let pos = self.stream.seek(StreamPos::Current(0))?;

        if pos == self.size {
            return Err(false);
        }

        let bytes_to_read = core::cmp::min(buf.len(), (self.size - pos) as usize);
        let mut bytes_read = 0;

        while bytes_read < bytes_to_read {
            if let Ok(read) = self.stream.read(&mut buf[bytes_read..bytes_to_read]) {
                bytes_read += read;
            } else if bytes_read == 0 {
                return Err(false);
            } else {
                break;
            }
        }

        Ok(bytes_read)
    }

    pub fn seek(&mut self, pos: StreamPos) -> Result<u32, bool> {
        self.stream.seek(pos)?;
        Ok(42)
    }

    pub fn close(self) {

    }
}

