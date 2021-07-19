use std::io::Read;
use std::fs::File;
use std::mem;
use std::slice;

mod bsa;
use bsa::PreHeader;
use bsa::v105;


fn main() {
    let buffer = File::open("./test.bsa").expect("file not found!");

    let pre_header: PreHeader = read_struct(&buffer);
    println!("found Version: {}", pre_header.version);
    
    if pre_header.version == 105 {
        
        let header: v105::Header = read_struct(&buffer);
        println!("Header: {:#?}", header);

        let mut dirs: Vec<v105::FolderRecord> = Vec::new();

        for _ in 0..header.folder_count {
            let dir: v105::FolderRecord = read_struct(&buffer);
            println!("Dir: {:#?}", dir.name_hash);
            dirs.push(dir);
        }

        let mut files: Vec<v105::FileRecord> = Vec::new();

        for _ in 0..header.file_count {
            let file: v105::FileRecord = read_struct(&buffer);
            println!("file: {:#?}", file.name_hash);
            files.push(file);
        }

        if header.archive_flags
            .contains(v105::ArchiveFlags::IncludeDirectoryNames) {
            
            
        }

    } else {
        println!("unknown version");
    }
}

fn read_struct<S>(mut f: &File) -> S {
    unsafe {
        let mut val: S = mem::zeroed();
        let slice = slice::from_raw_parts_mut(
            &mut val as *mut _ as *mut u8,
            mem::size_of::<S>()
        );
        // `read_exact()` comes from `Read` impl for `&[u8]`
        f.read_exact(slice).unwrap();
        
        val
    }
}
