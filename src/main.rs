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
        #[structopt(short, long)]
        attributes: bool,

        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::from_args();
    match args {
        Args::List{ file, attributes } => list(&file, attributes),
    }
}

fn list(file: &PathBuf, attributes: bool) -> Result<()> {
    let file = File::open(file).expect("file not found!");
    let mut buffer = BufReader::new(file);

    let version = Version::read(&mut buffer, ())?;
    match version {
        Version::V105 => {
            let header = v105::Header::read(&mut buffer, ())?;
            let dirs = v105::file_tree(&mut buffer, header)?;
            for dir in dirs {
                for file in dir.files {
                    if attributes {
                        let c = if file.compressed { "c" } else { " " };
                        println!("{0} {1: >8} {2}/{3}", c, file.size / 1000, dir.name, file.name);
                    } else {
                        println!("{0}/{1}", dir.name, file.name);
                    }
                }
            }
        },
        v => println!("Unsupported Version: {}", v),
    }
    Ok(())
}
