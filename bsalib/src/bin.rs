use std::{
    io::{Read, Write, Seek, SeekFrom, Result, Cursor},
    path,
    fs,
    mem::size_of,
    fmt,
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


pub trait Readable: Sized
where
    Self::Arg: Copy
{
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
default impl<T: Sized + fmt::Debug + Pod> Readable for T {
    fn read_here<R: Read + Seek>(reader: R, _: &Self::Arg) -> Result<Self> {
        read_struct(reader)
    }
}
impl Readable for u8  {}
impl Readable for u16 {}
impl Readable for u32 {}
impl Readable for u64 {}

pub trait Writable {
    fn size(&self) -> usize;

    fn write_here<W: Write>(&self, writer: W) -> Result<()>;
}
impl Writable for u8  {
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
impl<A: Writable> Writable for Vec<A> {
    fn size(&self) -> usize {
        self.into_iter()
            .map(|a| a.size())
            .sum()
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        for a in self {
            a.write_here(&mut out)?;
        }
        Ok(())
    }
}
impl<A: Writable> Writable for Option<A> {
    fn size(&self) -> usize {
        self.into_iter()
            .map(|a| a.size())
            .sum()
    }
    fn write_here<W: Write>(&self, mut out: W) -> Result<()> {
        for a in self {
            a.write_here(&mut out)?;
        }
        Ok(())
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