#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization, seek_stream_len)]
#[macro_use]
mod bin;
mod record;

pub use crate::record::*;


#[cfg(test)]
pub(crate) mod test {
    use std::fs::File;
    use std::io::Result;

    use super::*;

    #[test]
    fn load_unoffical_patch() -> Result<()> {
        let f = File::open("../test-data/unofficialSkyrimSEpatch.esp")?;
        let mut reader = EspReader::new(f);

        for entry in reader.top_level_entries()? {
            match entry {
                Entry::Record(r) => {
                    println!("Record: {}", r.record_type);
                },
                Entry::Group(g) => {
                    println!("Group: {}", g.group_info.label);
                }
            }
        }
        Ok(())
    }
}
