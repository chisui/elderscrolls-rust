use std::mem::size_of;
use std::io::{Read, ErrorKind, Result};
use bytemuck::Pod;


/// Reinterprets the next n bytes of the input as a struct,
/// where n is the size of the struct.
pub fn read_struct<S: Pod, R: Read>(mut reader: R) -> Result<(S, usize)> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok((val, size_of::<S>()))
}

pub fn read_to_end<T: Readable, R: Read>(mut reader: R, size: usize) -> Result<(Vec<T>, usize)> {
    let mut vals: Vec<T> = vec!();
    let mut bytes_left = size;
    while bytes_left > 0 {
        let (t, bytes) = T::read(&mut reader)?;
        vals.push(t);
        bytes_left -= bytes;
    }
    Ok((vals, size))
}

pub fn read_to_eof<T, R: Read>(mut reader: R) -> Result<(Vec<T>, usize)>
where T: Readable {
    let mut vals: Vec<T> = vec!();
    let mut size: usize = 0;
    loop {
        match T::read(&mut reader) {
            Ok((f, s)) => {
                vals.push(f);
                size += s;
            },
            Err(e) => if e.kind() == ErrorKind::UnexpectedEof {
                break
            } else {
                return Err(e)
            }
        }
    }
    Ok((vals, size))
}

pub trait Readable: Sized {
    fn read<R: Read>(reader: R) -> Result<(Self, usize)>;
}
