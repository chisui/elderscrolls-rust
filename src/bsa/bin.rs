use std::io::{Read, Seek, SeekFrom, Result};
use std::fmt::Debug;
use bytemuck::Pod;


pub fn read_struct<S: Pod, R: Read + Seek>(mut reader: R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

pub trait Readable: Sized + Debug
where <Self as Readable>::ReadableArgs: Copy {
    type ReadableArgs = ();

    fn offset(_: <Self as Readable>::ReadableArgs) -> Option<u64> {
        None
    }

    fn read<R: Read + Seek>(mut reader: R, args: <Self as Readable>::ReadableArgs) -> Result<Self> {
        match Self::offset(args) {
            Some(i) => reader.seek(SeekFrom::Start(i))?,
            _ => 0,
        };
        Self::read_here(reader, args)
    }

    fn read_here<R: Read + Seek>(reader: R, args: <Self as Readable>::ReadableArgs) -> Result<Self>;
    
    fn read_many<R: Read + Seek>(mut reader: R, num: usize, args: <Self as Readable>::ReadableArgs) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read(&mut reader, args)?;
            vals.push(val);
        }
        Ok(vals)
    }
}
