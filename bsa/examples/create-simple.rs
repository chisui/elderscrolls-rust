use std::path::PathBuf;
use std::fs::File;
use std::io::Result;

use bsa::*;

fn main() -> Result<()> {
    let dirs = [
        BsaDirSource::new("a", [
            BsaFileSource::new("b", b"some raw data")
        ])
    ];
    
    let writer = BsaWriterV105::default();
    let out = File::create("some.bsa")?;
    writer.write_bsa(dirs, out)
}
