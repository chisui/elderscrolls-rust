use bytemuck::Pod;
use std::io::{Read, Seek, BufReader, Result};
use std::fs::File;

mod bsa;
use bsa::version::Version;
use bsa::v105;


fn main() -> Result<()> {
    let file = File::open("./test.bsa").expect("file not found!");
    let buffer = BufReader::new(file);

    let version: Version = Version::read(buffer)?;

    println!("Version: {}", version);
    if version == Version::V105 {
        
        let header: v105::Header = read_struct::<v105::RawHeader, _>(&buffer)
            .map(v105::Header::from)?;
        println!("Header: {:#?}", header);

        // folder records
        let mut dirs: Vec<v105::FolderRecord> = Vec::new();
        for _ in 0..header.folder_count {
            let dir: v105::FolderRecord = read_struct(&buffer)?;
            dirs.push(dir);
        }

        println!("Header: {:#?}", dirs);
    }
    Ok(())
}

fn read_struct<S: Pod, R: Read>(mut reader: &R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}
