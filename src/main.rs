use std::io::{BufReader, Result};
use std::fs::File;
use std::collections::HashMap;

mod bsa;
use bsa::bin::read_struct;
use bsa::version::Version;
use bsa::v105;
use bsa::v105::ArchiveFlags::{IncludeDirectoryNames, IncludeFileNames};


fn main() -> Result<()> {
    let file = File::open("./test.bsa").expect("file not found!");
    let mut buffer = BufReader::new(file);

    let version: Version = Version::read(&mut buffer)?;

    println!("Version: {}", version);
    if version == Version::V105 {
        
        let header: v105::Header = read_struct::<v105::RawHeader, _>(&mut buffer)
            .map(v105::Header::from)?;
        println!("Header: {:#?}", header);

        // folder records
        let mut dirs: Vec<v105::FolderRecord> = Vec::new();
        for _ in 0..header.folder_count {
            let dir: v105::FolderRecord = read_struct(&mut buffer)?;
            dirs.push(dir);
        }

        let has_dir_name = header.has_archive_flag(IncludeDirectoryNames);
        let mut dir_contents = Vec::new();
        for dir in dirs {
            let dir_content = v105::FolderContentRecord::read(has_dir_name, dir.file_count, &mut buffer)?;
            dir_contents.push(dir_content);
        }
        let mut file_names: HashMap<v105::Hash, v105::BZString> = HashMap::with_capacity(header.file_count as usize);
        if header.has_archive_flag(IncludeFileNames) {
            for _ in 0..header.file_count {
                let name = v105::BZString::read_null_terminated(&mut buffer)?;
                file_names.insert(v105::Hash::from(&name), name);
            }
        }

        println!("names: {:#?}", file_names);

    }
    Ok(())
}
