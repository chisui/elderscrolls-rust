use std::io::{BufReader, Result};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use bsa::bin::Readable;
use bsa::version::Version;
use bsa::v105;
use bsa::v105::ArchiveFlags::{IncludeDirectoryNames, IncludeFileNames};


#[derive(Debug, StructOpt)]
#[structopt(about = "Bethesda Softworks Archive tool")]
enum Args {
    List {        
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::from_args();
    match args {
        Args::List{ file } => list(&file),
    }
}

fn list(file: &PathBuf) -> Result<()> {
    let file = File::open(file).expect("file not found!");
    let mut buffer = BufReader::new(file);

    let version: Version = Version::read(&mut buffer, ())?;

    println!("Version: {}", version);
    if version == Version::V105 {
        
        let header = v105::Header::read(&mut buffer, ())?;
        println!("Header: {:#?}", header);

        let dirs: Vec<v105::FolderRecord> = v105::FolderRecord::read_many(&mut buffer, header.folder_count as usize, ())?;

        let has_dir_name = header.has_archive_flag(IncludeDirectoryNames);
        let mut dir_contents = Vec::new();
        for dir in dirs {
            let dir_content = v105::FolderContentRecord::read(&mut buffer, (has_dir_name, dir.file_count))?;
            dir_contents.push(dir_content);
        }
        let file_names = if header.has_archive_flag(IncludeFileNames) {
            v105::FileNames::read(&mut buffer, header.file_count)?
        } else {
            v105::FileNames::empty()
        };

        println!("names: {:#?}", file_names);

    }
    Ok(())
}
