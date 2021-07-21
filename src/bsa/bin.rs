use std::io::{Read, Seek, Result};
use bytemuck::Pod;


pub fn read_struct<S: Pod, R: Read + Seek>(mut reader: R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

pub trait Readable: Sized
where <Self as Readable>::ReadableArgs: Copy {
    type ReadableArgs;
    fn read<R: Read + Seek>(reader: R, args: <Self as Readable>::ReadableArgs) -> Result<Self>;
    
    fn read_many<R: Read + Seek>(mut reader: R, num: usize, args: <Self as Readable>::ReadableArgs) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read(&mut reader, args)?;
            vals.push(val);
        }
        Ok(vals)
    }
}
