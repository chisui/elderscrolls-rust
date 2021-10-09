use std::path::PathBuf;
use std::env;
use std::io::Result;

use bsa::*;

fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    let mut bsa: SomeBsaReader<_> = bsa::open(file)?;

    match bsa.list()? {
        SomeBsaRoot::V001(files) => {
            for file in files {
                println!("{}", &file.id);
            }
        },
        SomeBsaRoot::V10X(dirs) => {
            for dir in dirs {
                for file in &dir {
                    println!("{}\\{}", &dir.id, &file.id);
                }
            }
        },
    }
    Ok(())
}

