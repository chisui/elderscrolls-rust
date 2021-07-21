use std::io::{BufReader, Result};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

use bsa::open::WrappedBsa;
use bsa::open::WrappedBsa::V105;


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

    let some_bsa = WrappedBsa::open(&mut buffer)?;
    match some_bsa {
        V105(mut bsa) => {
            println!("Header: {:#?}", bsa.header);
            let file_names = bsa.file_names()?;

            println!("names: {:#?}", file_names);
        }
    }
    Ok(())
}
