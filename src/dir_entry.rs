
pub struct DirEntry {
    pub sfn_buf: [u8; 12],
    pub sfn_len: usize,
    pub lfn_buf: [u8; 256],
    pub lfn_len: usize,
    pub attr: u8,
    pub cluster: u32,
    pub size: u32,
}

//const ATTR_READ_ONLY: u8 = 0x01;
//const ATTR_HIDDEN: u8 = 0x02;
//const ATTR_SYSTEM: u8 = 0x04;
//const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
//const ATTR_ARCHIVE: u8 = 0x20;
//const ATTR_LONG_FILE_NAME: u8 = 0x0f;

impl DirEntry{
    pub fn root(cluster: u32) -> Self {
        DirEntry {
            sfn_buf: [0u8; 12],
            sfn_len: 0,
            lfn_buf: [0u8; 256],
            lfn_len: 0,
            attr: ATTR_DIRECTORY,
            cluster,
            size: 0,
        }
    }

    pub fn compare(&self, name: &[u8]) -> bool {
        if name.len() == self.sfn_len {
            let mut equal = true;

            for (i, &c) in name.iter().enumerate() {
                if self.sfn_buf[i] != c {
                    equal = false;
                    break;
                }
            }

            if equal {
                return true;
            }
        }

        if name.len() == self.lfn_len {
            for (i, &c) in name.iter().enumerate() {
                if self.lfn_buf[i] != c {
                    return false;
                }
            }

            return true;
        }

        false
        /*
        let current_name = self.name();

        if name.len() != current_name.len() {
            return false;
        }

        for (i, &c) in name.iter().enumerate() {
            if current_name[i] != c {
                return false;
            }
        }
        true
        */
    }
    /*
    pub fn compare(&self, name: &[u8]) -> bool {
        let current_name = self.name();

        if name.len() != current_name.len() {
            return false;
        }

        for (i, &c) in name.iter().enumerate() {
            if current_name[i] != c {
                return false;
            }
        }
        true
    }
    */

    /*
    pub fn find(&self, name: &[u8]) -> Option<DirEntry<T>> {
        for dir_entry in self.read_dir() {
            if dir_entry.compare(name) {
                return Some(dir_entry);
            }
        }
        None
    }
    */
    pub fn name(&self) -> &[u8] {
        if self.lfn_len > 0 {
            &self.lfn_buf[..self.lfn_len]
        } else {
            &self.sfn_buf[..self.sfn_len]
        }
    }
    /*
    pub fn open_file(&self) -> Result<File<T>, bool> {
        if self.is_file() {
            Ok(File::new(Stream::new(self.fs, self.cluster), self.size()))
        } else {
            Err(false)
        }
    }

    pub fn read_dir(&self) -> DirReader<'a, T> {
        DirReader::new(self.fs, self.cluster)
    }
    */
    pub fn is_dir(&self) -> bool {
        (self.attr & ATTR_DIRECTORY) != 0
    }

    pub fn is_file(&self) -> bool {
        (self.attr & ATTR_DIRECTORY) == 0
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

