pub struct Path<'a> {
    path: &'a [u8],
    pos: usize,
}

impl <'a> Path<'a> {
    pub fn new(path: &str) -> Path {
        Path {
            path: path.as_bytes(),
            pos: 0,
        }
    }
}

impl <'a>Iterator for Path<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let start_pos = self.pos;
        let mut stop_pos = self.pos;

        while stop_pos < self.path.len() {
            if self.path[stop_pos] == b'/' || self.path[stop_pos] == b'\\' {
                break;
            }

            stop_pos += 1;
        }

        if start_pos == stop_pos {
            None
        } else {
            self.pos = stop_pos + 1;
            Some(&self.path[start_pos..stop_pos])
        }
    }
}