use std::io::{BufReader, Result};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use bsa::bin::Readable;
use bsa::version::Version;
use bsa::v105;


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

    let version = Version::read(&mut buffer, ())?;
    match version {
        Version::V105 => {
            let header = v105::Header::read(&mut buffer, ())?;
            let dirs = v105::file_tree(&mut buffer, header)?;
            for dir in dirs {
                for file in dir.files {
                    println!("{}/{} {}", dir.name, file.name, if file.compressed {
                        "compressed"
                    } else {
                        "uncompressed"
                    });
                }
            }
        },
        v => println!("Unsupported Version: {}", v),
    }
    Ok(())
}
