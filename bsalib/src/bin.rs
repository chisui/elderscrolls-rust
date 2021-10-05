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


pub trait Readable: Sized {
    type Arg = ();

    fn offset0() -> Option<usize> 
    where Self::Arg: Default {
        Self::offset(&Self::Arg::default())
    }

    fn offset(_: &Self::Arg) -> Option<usize> {
        None
    }

    fn read0<R: Read + Seek>(reader: R) -> Result<Self>
    where Self::Arg: Default {
        Self::read(reader, &Self::Arg::default())
    }

    fn read<R: Read + Seek>(mut reader: R, args: &Self::Arg) -> Result<Self> {
        if let Some(i) = Self::offset(args) {
            reader.seek(SeekFrom::Start(i as u64))?;
        }
        Self::read_here(&mut reader, args)
    }

    fn read_here<R: Read + Seek>(reader: R, args: &Self::Arg) -> Result<Self>;
    
    fn read_here0<R: Read + Seek>(reader: R) -> Result<Self>
    where Self::Arg: Default {
        Self::read_here(reader, &Self::Arg::default())
    }

    fn read_many0<R: Read + Seek>(reader: R, num: usize) -> Result<Vec<Self>>
    where Self::Arg: Default {
        Self::read_many(reader, num, &Self::Arg::default())
    }

    fn read_many<R: Read + Seek>(mut reader: R, num: usize, args: &Self::Arg) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read_here(&mut reader, args)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

#[macro_export]
macro_rules! derive_readable_via_pod {
    ( $t:ty ) => {
        impl crate::bin::Readable for $t {
            fn read_here<R: std::io::Read + std::io::Seek>(reader: R, _: &<Self as crate::bin::Readable>::Arg) -> std::io::Result<Self> {
                crate::bin::read_struct(reader)
            }
        }
    };
}
derive_readable_via_pod!(u8);
derive_readable_via_pod!(u16);
derive_readable_via_pod!(u32);
derive_readable_via_pod!(u64);

pub trait Writable {
    fn size(&self) -> usize;

    fn write_here<W: Write>(&self, writer: W) -> Result<()>;
}

#[macro_export]
macro_rules! derive_writable_via_pod {
    ( $t:ty ) => {
        impl crate::bin::Writable for $t {
            fn size(&self) -> usize { std::mem::size_of::<Self>() }
            fn write_here<W: std::io::Write>(&self, writer: W) -> std::io::Result<()> {
                crate::bin::write_struct(self, writer)
            }
        }
    };
}
derive_writable_via_pod!(u8);
derive_writable_via_pod!(u16);
derive_writable_via_pod!(u32);
derive_writable_via_pod!(u64);

#[macro_export]
macro_rules! derive_writable_via_into_iter {
    ( $t:tt ) => {
        impl<A: Writable> Writable for $t<A> {
            fn size(&self) -> usize {
                self.into_iter()
                    .map(|a| a.size())
                    .sum()
            }
            fn write_here<W: std::io::Write>(&self, mut out: W) -> std::io::Result<()> {
                for a in self {
                    a.write_here(&mut out)?;
                }
                Ok(())
            }
        }
    };
}
derive_writable_via_into_iter!(Vec);
derive_writable_via_into_iter!(Option);

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

pub struct Positioned<A> {
    pub position: u64,
    pub data: A,
}
impl<A: Writable> Positioned<A> {
    
    pub fn new<W: Write + Seek>(data: A, mut out: W) -> Result<Self> {
        let position = out.stream_position()?;
        data.write_here(&mut out)?;
        Ok(Self { position, data })
    }

    pub fn new_empty<W: Write + Seek>(out: W) ->  Result<Self> 
    where A: Default {
        Self::new(A::default(), out)
    }

    pub fn update<W: Write + Seek>(&mut self, mut out: W) -> Result<()> {
        let tmp_pos = out.stream_position()?;
        out.seek(SeekFrom::Start(self.position))?;
        self.data.write_here(&mut out)?;
        out.seek(SeekFrom::Start(tmp_pos))?;
        Ok(())
    }

    pub fn map<F, W>(&mut self, out: W, f: F) -> Result<()>
    where
        F: FnOnce(&A) -> Result<A>,
        W: Write + Seek
    {
        self.data = f(&self.data)?;
        self.update(out)
    }
}
impl<A: Readable> Readable for Positioned<A> {
    type Arg = A::Arg;
    fn read_here<R: Read + Seek>(mut reader: R, arg: &A::Arg) -> Result<Self> {
        let position = reader.stream_position()?;
        let data = A::read_here(reader, arg)?;
        Ok(Positioned { position, data })
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
    
    pub fn write_read_identity<A: Writable + Readable<Arg = ()> + Debug + Eq>(expected: A) {
        let actual = write_read(&expected);

        assert_eq!(expected, actual)
    }

    pub fn write_read<A: Writable + Readable<Arg = ()> + Debug>(val: &A) -> A {
        let mut out = Cursor::new(Vec::<u8>::new());
        val.write_here(&mut out)
            .unwrap_or_else(|err| panic!("could not write {:?}: {}", val, err));
        let mut input = Cursor::new(out.into_inner());
        A::read_here0(&mut input)
            .unwrap_or_else(|err| panic!("could not read {:?}: {}", val, err))
    }
}
