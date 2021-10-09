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
    let mut bsa: SomeReaderV10X<_> = bsa::open(file)?;

    for dir in bsa.list()? {
        for file in &dir {
            println!("{}\\{}", &dir.id, &file.id);
        }
    }
    Ok(())
}

