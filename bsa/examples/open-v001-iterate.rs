use std::path::PathBuf;
use std::env;
use std::io::Result;

use bsa::*;

fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    println!("{:?}", file);
    let mut bsa: BsaReaderV001<_> = bsa::open(file)?;

    for file in bsa.list()? {
        println!("{}", &file.id);
    }
    Ok(())
}

