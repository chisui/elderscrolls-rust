use std::path::PathBuf;
use std::env;
use std::io::{self, Result};

use bsa::*;


fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    let mut bsa: ReaderV105<_> = bsa::open(file)?;

    let dirs = bsa.list()?;
    let file = &dirs[0][0];
    bsa.extract(&file, io::stdout())
}

