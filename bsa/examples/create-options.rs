use std::path::PathBuf;
use std::fs::File;
use std::io::Result;

use bsa::*;

fn main() -> Result<()> {
    let dirs = [
        BsaDirSource::new("a", [
            BsaFileSource::new("b", PathBuf::from("some-file"))
        ])
    ];
    
    let writer = BsaWriterV105::new(
        [ArchiveFlagV105::CompressedArchive, ArchiveFlagV105::EmbedFileNames],
        [FileFlag::Miscellaneous],
    );
    let out = File::create("some.bsa")?;
    writer.write_bsa(dirs, out)
}
