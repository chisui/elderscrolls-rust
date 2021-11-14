use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::io::{Error, ErrorKind, Result};

use esp::*;


fn main() -> Result<()> {
    let file = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap();

    let f = File::open(file)?;
    let entries = read_esp(f).map_err(|err| Error::new(ErrorKind::Other, format!("{}", err)))?;
    print_entries(0, entries);
    Ok(())
}
fn print_entries(indent: usize, entries: Vec<Entry>) {
    for e in entries {
        match e {
            Entry::Record(rec) => {
                match rec {
                    SomeRecord::Other(t, _) => println!("{}{}", "  ".repeat(indent), t),
                    _ => println!("{}{:?}", "  ".repeat(indent), rec),
                }            
            },
            Entry::Group(grp) => {
                println!("{}{}", "  ".repeat(indent), grp.info);
                print_entries(indent + 1, grp.entries);
            },
        }
    }
}
