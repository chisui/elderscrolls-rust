use std::path::PathBuf;
use std::env;
use std::io::Result;

use bsa::*;

fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    let mut bsa: SomeReader<_> = bsa::open(file)?;

    match bsa.list()? {
        SomeRoot::V001(files) => {
            for file in files {
                println!("{}", &file.id);
            }
        },
        SomeRoot::V10X(dirs) => {
            for dir in dirs {
                for file in &dir {
                    println!("{}\\{}", &dir.id, &file.id);
                }
            }
        },
    }
    Ok(())
}

