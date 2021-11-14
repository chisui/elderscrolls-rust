use std::io::{self, BufRead, Read};

use bytemuck::Pod;


pub trait ReadStructExt<S>
where S: Sized {
    fn read_struct(&mut self) -> io::Result<S>;
}

impl<S, R> ReadStructExt<S> for R
where S: Pod, R: Read {
    fn read_struct(&mut self) -> io::Result<S> {
        let mut val = S::zeroed();
        let slice = bytemuck::bytes_of_mut(&mut val);
        self.read_exact(slice)?;
        Ok(val)
    }
}

pub trait Readable<R>: Sized
where Self::Error: From<io::Error> {
    type Error = io::Error;
    fn read_val(reader: &mut R) -> Result<Self, Self::Error>;
}

impl<R: Read> Readable<R> for u8 {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}
impl<R: Read> Readable<R> for u16 {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}
impl<R: Read> Readable<R> for u32 {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}
impl<R: Read> Readable<R> for u64 {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}
impl<T, R> Readable<R> for Vec<T>
where T: Readable<R>, R: BufRead {
    type Error = T::Error;
    fn read_val(reader: &mut R) -> Result<Self, Self::Error> {
        let mut values = Vec::new();
        while reader.has_data_left()? {
            let val = T::read_val(reader)?;
            values.push(val);
        }
        Ok(values)
    }
}
