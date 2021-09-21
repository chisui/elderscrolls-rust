use std::io::{Read, Write, Seek, SeekFrom, Result};
use std::mem::size_of;
use std::fmt;
use bytemuck::Pod;

pub fn read_struct<S: Pod, R: Read>(mut reader: R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

pub fn write_struct<S: Pod, W: Write>(val: &S, mut writer: W) -> Result<()> {
    let bytes = bytemuck::bytes_of(val);
    writer.write_all(bytes)
}


pub trait Readable: Sized + fmt::Debug
where <Self as Readable>::ReadableArgs: Copy {
    type ReadableArgs = ();

    fn offset0() -> Option<usize> 
    where Self::ReadableArgs: Default {
        Self::offset(&Self::ReadableArgs::default())
    }

    fn offset(_: &<Self as Readable>::ReadableArgs) -> Option<usize> {
        None
    }

    fn read0<R: Read + Seek>(reader: R) -> Result<Self>
    where Self::ReadableArgs: Default {
        Self::read(reader, &Self::ReadableArgs::default())
    }

    fn read<R: Read + Seek>(mut reader: R, args: &<Self as Readable>::ReadableArgs) -> Result<Self> {
        if let Some(i) = Self::offset(args) {
            reader.seek(SeekFrom::Start(i as u64))?;
        }
        Self::read_here(&mut reader, args)
    }

    fn read_here<R: Read + Seek>(reader: R, args: &<Self as Readable>::ReadableArgs) -> Result<Self>;
    
    fn read_here0<R: Read + Seek>(reader: R) -> Result<Self>
    where Self::ReadableArgs: Default {
        Self::read_here(reader, &Self::ReadableArgs::default())
    }

    fn read_many0<R: Read + Seek>(reader: R, num: usize) -> Result<Vec<Self>>
    where Self::ReadableArgs: Default {
        Self::read_many(reader, num, &Self::ReadableArgs::default())
    }

    fn read_many<R: Read + Seek>(mut reader: R, num: usize, args: &<Self as Readable>::ReadableArgs) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read_here(&mut reader, args)?;
            vals.push(val);
        }
        Ok(vals)
    }
}
default impl<T: Sized + fmt::Debug + Pod> Readable for T {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::ReadableArgs) -> Result<Self> {
        read_struct(reader)
    }
}
impl Readable for u8 {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::ReadableArgs) -> Result<Self> {
        read_struct(reader)
    }
}
impl Readable for u16 {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::ReadableArgs) -> Result<Self> {
        read_struct(reader)
    }
}
impl Readable for u32 {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::ReadableArgs) -> Result<Self> {
        read_struct(reader)
    }
}
impl Readable for u64 {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::ReadableArgs) -> Result<Self> {
        read_struct(reader)
    }
}

pub trait Writable {
    fn size(&self) -> usize;

    fn write_here<W: Write>(&self, writer: W) -> Result<()>;
}
default impl<T: Sized + fmt::Debug + Pod> Writable for T {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        write_struct(self, writer)
    }
}
impl Writable for u8 {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        write_struct(self, writer)
    }
}
impl Writable for u16 {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        write_struct(self, writer)
    }
}
impl Writable for u32 {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        write_struct(self, writer)
    }
}
impl Writable for u64 {
    fn size(&self) -> usize { size_of::<Self>() }
    fn write_here<W: Write>(&self, writer: W) -> Result<()> {
        write_struct(self, writer)
    }
}

pub fn size_many<I: IntoIterator>(vals: I) -> usize
where I::Item: Writable {
    vals.into_iter().map(|val| val.size()).sum()
}

pub fn write_many<I: IntoIterator, W: Write>(vals: I, mut writer: W) -> Result<()>
where I::Item: Writable {
    for val in vals {
        val.write_here(&mut writer)?;
    }
    Ok(())
}

pub const fn concat_bytes([a, b, c, d]: [u8; 4]) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}