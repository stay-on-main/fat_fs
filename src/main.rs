mod fs;
mod image;

use fs::Fs;
use fs::storage_io::StorageIo;
use fs::stream::StreamPos;
use fs::dir::Dir;
//use fs::dir_reader::DirEntry;
use std::fs::File;
use std::io::prelude::*;

fn print_str(s: &[u8]) {
    for c in s {
        print!("{}", *c as char);
    }

    println!();
}

fn print_tree<T: StorageIo>(fs: &Fs<T> ,dir: Dir<T>, level: usize) {
    for dir_entry in dir {

        if dir_entry.name()[0] != b'.' {
            for _ in 0..level {
                print!("    ");
            }

            print_str(dir_entry.name());
        }

        if dir_entry.is_dir() && dir_entry.name()[0] != b'.' {
            print_tree(fs, Dir::new(fs, dir_entry.cluster), level + 1);
        }
    }
}

fn main() {
    let images = vec!("fat32", "fat16", "fat12");
    
    for image in images {
        let mut path = String::from("F:/stay-on-main/");
        path.push_str(image);
        path.push_str(".img");
        let path_str = path.as_str();

        let img = image::new(path_str);
        let fs = Fs::new(img).unwrap();
        print_tree(&fs, fs.root_dir(), 0);

        //let mut file = fs.file_open("DEEPFO~1/DIPPER~1/IM-92700.JPG").unwrap();
        let mut file = fs.file_open("DeepFolder/dipper_folder/im-92700.jpg").unwrap();
        let mut out_path = String::from("C:/stay-on-main/xxx/target/debug/");
        out_path.push_str(image);
        out_path.push_str("_img.png");
        let mut out_file = File::create(out_path.as_str()).unwrap();
        let mut buf = [0u8; 40];

        while let Ok(bytes) = file.read(&mut buf) {
            //print_str(&buf[..bytes]);
            out_file.write(&buf[..bytes]).unwrap();
        }

        file.close();
    }
    
    /*
    let img = image::new("F:/stay-on-main/fat32.img");
    let fs = Fs::new(img).unwrap();
    print_tree(&fs, fs.root_dir(), 0);

    let img = image::new("F:/stay-on-main/fat16.img");
    let fs = Fs::new(img).unwrap();
    print_tree(&fs, fs.root_dir(), 0);
    
    let img = image::new("F:/stay-on-main/fat12.img");
    let fs = Fs::new(img).unwrap();
    print_tree(&fs, fs.root_dir(), 0);
    */
}
