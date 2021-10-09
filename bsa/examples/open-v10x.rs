use std::path::PathBuf;
use std::env;
use std::io::Result;

use bsa::*;


fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    let bsa: SomeReaderV10X<_> = bsa::open(file)?;

    println!("{:?}", bsa.version());
    println!("{:?}", bsa.header());
    Ok(())
}

