use std::path::PathBuf;
use std::fs::File;
use std::io::Result;

use bsa::*;
use bsa::write::*;


fn main() -> Result<()> {
    let dirs = [
        Dir::new("a", [
            File::new("b", b"some raw data")
        ])
    ];
    
    let writer = WriterV105::default();
    let out = File::create("some.bsa")?;
    writer.write_bsa(dirs, out)
}
