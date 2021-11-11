use std::io::{Read, Result};

use bytemuck::Pod;


pub fn read_struct<S: Pod, R: Read>(reader: &mut R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

pub trait Readable<R>: Sized {
    fn read(reader: &mut R) -> Result<Self>; 
}
