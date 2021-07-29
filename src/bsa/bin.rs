use std::io::{Read, Seek, SeekFrom, Result, Error, ErrorKind};
use std::fmt;
use std::error;
use bytemuck::Pod;


pub fn read_struct<S: Pod, R: Read>(mut reader: R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

#[derive(Debug)]
struct PositionedError(pub Error, pub u64);
impl error::Error for PositionedError {}
impl fmt::Display for PositionedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at: {:08}", self.0, self.1)
    }
}

pub trait Readable: Sized + fmt::Debug
where <Self as Readable>::ReadableArgs: Copy {
    type ReadableArgs = ();

    fn offset(_: &<Self as Readable>::ReadableArgs) -> Option<usize> {
        None
    }

    fn read<R: Read + Seek>(mut reader: R, args: &<Self as Readable>::ReadableArgs) -> Result<Self> {
        match Self::offset(args) {
            Some(i) => reader.seek(SeekFrom::Start(i as u64))?,
            _ => 0,
        };
        match Self::read_here(&mut reader, args) {
            Ok(v) => Ok(v),
            Err(e) => {
                let pos = reader.stream_position()?;
                err(PositionedError(e, pos))
            },
        }        
    }

    fn read_here<R: Read + Seek>(reader: R, args: &<Self as Readable>::ReadableArgs) -> Result<Self>;
    
    fn read_many<R: Read + Seek>(mut reader: R, num: usize, args: &<Self as Readable>::ReadableArgs) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read(&mut reader, args)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

pub fn err<E, R>(error: E) -> Result<R> 
where E: Into<Box<dyn error::Error + Send + Sync>> {
    Err(Error::new(ErrorKind::InvalidData, error))
}
