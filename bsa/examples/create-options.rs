use std::path::PathBuf;
use std::fs;
use std::io::Result;

use bsa::*;
use bsa::write::*;


fn main() -> Result<()> {
    let dirs = [
        Dir::new("a", [
            File::new("b", PathBuf::from("some-file"))
        ])
    ];
    
    let writer = WriterV105::new(
        [ArchiveFlagV105::CompressedArchive, ArchiveFlagV105::EmbedFileNames],
        [FileFlag::Miscellaneous],
    );
    let out = fs::File::create("some.bsa")?;
    writer.write_bsa(dirs, out)
}
