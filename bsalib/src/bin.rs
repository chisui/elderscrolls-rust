use std::{
    io::{Read, Write, Seek, SeekFrom, Result, Cursor},
    path,
    fs
};
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


pub trait Fixed {
    fn pos() -> usize;

    fn move_to_start<S: Seek>(mut seek: S) -> Result<()> {
        seek.seek(SeekFrom::Start(Self::pos() as u64))?;
        Ok(())
    }
}
pub trait VarSize {
    fn size(&self) -> usize;
}
macro_rules! derive_var_size_via_size_of {
    ( $t:ty ) => {
        impl crate::bin::VarSize for $t {
            fn size(&self) -> usize {
                std::mem::size_of::<Self>()
            }
        }
    };
}
pub(crate) use derive_var_size_via_size_of;
derive_var_size_via_size_of!(u8);
derive_var_size_via_size_of!(u16);
derive_var_size_via_size_of!(u32);
derive_var_size_via_size_of!(u64);
impl<A: VarSize> VarSize for Vec<A> {
    fn size(&self) -> usize {
        self.iter().map(A::size).sum()
    }
}
impl<A: VarSize> VarSize for Option<A> {
    fn size(&self) -> usize {
        self.iter().map(A::size).sum()
    }
}
pub trait ReadableFixed: Sized {
    fn read_fixed<R: Read + Seek>(reader: R) -> Result<Self>;
}
pub fn read_fixed_default<A: Fixed + Pod, R: Read + Seek>(mut reader: R) -> Result<A> {
    A::move_to_start(&mut reader)?;
    read_struct(reader)
}
pub trait Readable: Sized {
    fn read<R: Read>(reader: R) -> Result<Self>;

    fn read_many<R: Read>(mut reader: R, num: usize) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read(&mut reader)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

pub trait ReadableParam<P>: Sized {
    fn read<R: Read>(reader: R, param: P) -> Result<Self>;

    fn read_many<R: Read + Seek>(mut reader: R, num: usize, param: P) -> Result<Vec<Self>>
    where P: Copy {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read(&mut reader, param)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

macro_rules! derive_readable_via_pod {
    ( $t:ty ) => {
        impl crate::bin::Readable for $t {
            fn read<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
                crate::bin::read_struct(reader)
            }
        }
    };
}
pub(crate) use derive_readable_via_pod;
derive_readable_via_pod!(u8);
derive_readable_via_pod!(u16);
derive_readable_via_pod!(u32);
derive_readable_via_pod!(u64);

pub trait WritableFixed: Fixed {
    fn write_fixed<W: Write + Seek>(&self, writer: W) -> Result<()>;
}
pub fn write_fixed_default<A: Fixed + Pod, R: Write + Seek>(val: &A, mut writer: R) -> Result<()> {
    A::move_to_start(&mut writer)?;
    write_struct(val, writer)
}
pub trait Writable {
    fn write<W: Write>(&self, writer: W) -> Result<()>;
}
impl<A: Writable> Writable for [A] {
    fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        for val in self {
            val.write(&mut writer)?;
        }
        Ok(())
    }
}

macro_rules! derive_writable_via_pod {
    ( $t:ty ) => {
        impl crate::bin::Writable for $t {
            fn write<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
                crate::bin::write_struct(self, writer)
            }
        }
    };
}
pub(crate) use derive_writable_via_pod;
derive_writable_via_pod!(u8);
derive_writable_via_pod!(u16);
derive_writable_via_pod!(u32);
derive_writable_via_pod!(u64);
macro_rules! derive_writable_via_into_iter {
    ( $t:tt ) => {
        impl<A: Writable> Writable for $t<A> {
            fn write<W: std::io::Write>(&self, mut out: W) -> std::io::Result<()> {
                for a in self {
                    a.write(&mut out)?;
                }
                Ok(())
            }
        }
    };
}
pub(crate) use derive_writable_via_into_iter;
derive_writable_via_into_iter!(Vec);
derive_writable_via_into_iter!(Option);

pub const fn concat_bytes([a, b, c, d]: [u8; 4]) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

pub struct Positioned<A> {
    pub position: u64,
    pub data: A,
}
impl<A: Writable> Positioned<A> {
    
    pub fn new<W: Write + Seek>(data: A, mut out: W) -> Result<Self> {
        let position = out.stream_position()?;
        data.write(&mut out)?;
        Ok(Self { position, data })
    }

    pub fn new_empty<W: Write + Seek>(out: W) ->  Result<Self> 
    where A: Default {
        Self::new(A::default(), out)
    }

    pub fn update<W: Write + Seek>(&mut self, mut out: W) -> Result<()> {
        let tmp_pos = out.stream_position()?;
        out.seek(SeekFrom::Start(self.position))?;
        self.data.write(&mut out)?;
        out.seek(SeekFrom::Start(tmp_pos))?;
        Ok(())
    }
}

pub trait DataSource
where Self::Read: Read {
    type Read;
    fn open(&self) -> Result<Self::Read>;
}
impl DataSource for path::Path {
    type Read = fs::File;
    fn open(&self) -> Result<Self::Read> {
        fs::File::open(self)
    }
}
impl DataSource for path::PathBuf {
    type Read = fs::File;
    fn open(&self) -> Result<Self::Read> {
        fs::File::open(self)
    }
}
impl DataSource for &[u8] {
    type Read = Cursor<Vec<u8>>;
    fn open(&self) -> Result<Cursor<Vec<u8>>> {
        self.to_vec().open()
    }
}
impl DataSource for Vec<u8> {
    type Read = Cursor<Vec<u8>>;
    fn open(&self) -> Result<Cursor<Vec<u8>>> {
        Ok(Cursor::new(self.to_vec()))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::fmt::Debug;
    use super::*;
    
    pub fn write_read_identity<A: Writable + Readable + Debug + Eq>(expected: A) {
        let actual = write_read(&expected);

        assert_eq!(expected, actual)
    }

    pub fn write_read<A: Writable + Readable + Debug>(val: &A) -> A {
        let mut out = Cursor::new(Vec::<u8>::new());
        val.write(&mut out)
            .unwrap_or_else(|err| panic!("could not write {:?}: {}", val, err));
        let mut input = Cursor::new(out.into_inner());
        A::read(&mut input)
            .unwrap_or_else(|err| panic!("could not read {:?}: {}", val, err))
    }

    pub fn write_read_fixed_identity<A: WritableFixed + ReadableFixed + Debug + Eq>(expected: A) {
        let actual = write_read_fixed(&expected);

        assert_eq!(expected, actual)
    }

    pub fn write_read_fixed<A: WritableFixed + ReadableFixed + Debug>(val: &A) -> A {
        let mut out = Cursor::new(Vec::<u8>::new());
        val.write_fixed(&mut out)
            .unwrap_or_else(|err| panic!("could not write {:?}: {}", val, err));
        let mut input = Cursor::new(out.into_inner());
        A::read_fixed(&mut input)
            .unwrap_or_else(|err| panic!("could not read {:?}: {}", val, err))
    }
}
