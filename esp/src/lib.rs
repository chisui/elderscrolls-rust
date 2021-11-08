#![allow(incomplete_features)]
#![feature(associated_type_defaults, wrapping_int_impl, specialization)]
#[macro_use]
mod bin;
mod record;

pub use crate::record::{EntryType, RecordType, GenericRecord};


#[cfg(test)]
pub(crate) mod test {
    use std::{fs::File, io::Result};
    use crate::bin::{Readable, ReadableParam};

    use super::*;

    #[test]
    fn load_unoffical_patch() -> Result<()> {
        let mut f = File::open("../test-data/unofficialSkyrimSEpatch.esp")?;

        let t = EntryType::read_bin(&mut f)?;
        println!("{:?}", t);
        if let EntryType::Record(rt) = t {
            let r = GenericRecord::read_with_param(&mut f, rt)?;
            println!("{:#?}", r);
        }

        Ok(())
    }
}