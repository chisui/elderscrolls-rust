use std::{
    io::{Read, Write, Seek, SeekFrom, Result, Cursor},
    path,
    fs
};
use bytemuck::Pod;


/// Reinterprets the next n bytes of the input as a struct,
/// where n is the size of the struct.
pub fn read_struct<S: Pod, R: Read>(mut reader: R) -> Result<S> {
    let mut val = S::zeroed();
    let slice = bytemuck::bytes_of_mut(&mut val);
    reader.read_exact(slice)?;
    Ok(val)
}

/// Reinterprets the struct as bytes and writes them to the output.
pub fn write_struct<S: Pod, W: Write>(val: &S, mut writer: W) -> Result<()> {
    let bytes = bytemuck::bytes_of(val);
    writer.write_all(bytes)
}

/// A struct with a fixed position inside of a bytestream.
pub trait Fixed: Sized {

    /// The position of the struct.
    fn pos() -> usize;

    /// Move to the start of this structs position.
    fn move_to_start<S: Seek>(mut seek: S) -> Result<()> {
        seek.seek(SeekFrom::Start(Self::pos() as u64))?;
        Ok(())
    }
}

/// A struct that has a known size at runtime.
/// This may also be implemented for structs that are Sized.
pub trait VarSize {

    /// The current size in bytes of the struct.
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
impl<A: VarSize> VarSize for [A] {
    fn size(&self) -> usize {
        self.iter().map(A::size).sum()
    }
}
impl<A: VarSize> VarSize for &[A] {
    fn size(&self) -> usize {
        self.iter().map(A::size).sum()
    }
}
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

/// A struct that can be Read at a fixed size
pub trait ReadableFixed: Fixed {

    /// Read the struct from the input.
    /// After this operation the input should be at the first byte after the
    /// read struct.
    fn read_fixed<R: Read + Seek>(reader: R) -> Result<Self>;
}

/// default implementation for read_fixed using Fixed and read_struct.
pub fn read_fixed_default<A: Fixed + Pod, R: Read + Seek>(mut reader: R) -> Result<A> {
    A::move_to_start(&mut reader)?;
    read_struct(reader)
}
macro_rules! derive_readable_fixed_via_default {
    ( $t:ty ) => {
        impl crate::bin::ReadableFixed for $t {
            fn read_fixed<R: std::io::Read + std::io::Seek>(reader: R) -> std::io::Result<Self> {
                crate::bin::read_fixed_default(reader)
            }
        }
    };
}

/// A struct that can be read from a bytestream.
pub trait Readable: VarSize + Sized {

    /// Read the struct from the stream.
    /// This should consume exactly the number of bytes that [`VarSize::size()`] returns.
    fn read_bin<R: Read>(reader: R) -> Result<Self>;

    /// Read multiple of the struct from a stream.
    /// This should consume exactly the number of bytes that [`VarSize::size()`] returns
    /// for each of the elements.
    fn read_bin_many<R: Read>(mut reader: R, num: usize) -> Result<Vec<Self>> {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read_bin(&mut reader)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

/// A struct that can be read from a bytestream but needs additional parameters
/// to be read.
pub trait ReadableParam<P>: VarSize + Sized {
    
    /// Read the struct from the stream.
    /// This should consume exactly the number of bytes that [`VarSize::size()`] returns.
    fn read_with_param<R: Read>(reader: R, param: P) -> Result<Self>;

    /// Read multiple of the struct from a stream.
    /// This should consume exactly the number of bytes that [`VarSize::size()`] returns
    /// for each of the elements.
    fn read_with_param_many<R: Read + Seek>(mut reader: R, num: usize, param: P) -> Result<Vec<Self>>
    where P: Copy {
        let mut vals = Vec::new();
        for _ in 0..num {
            let val = Self::read_with_param(&mut reader, param)?;
            vals.push(val);
        }
        Ok(vals)
    }
}

macro_rules! derive_readable_via_pod {
    ( $t:ty ) => {
        impl crate::bin::Readable for $t {
            fn read_bin<R: std::io::Read>(reader: R) -> std::io::Result<Self> {
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


/// A struct that can be read from a bytestream at a fixed position.
pub trait WritableFixed: Fixed + VarSize {

    /// Write the struct.
    /// After this operation the output should be at the first byte after the
    /// writen struct.
    fn write_fixed<W: Write + Seek>(&self, writer: W) -> Result<()>;
}

/// default implementation for write_fixed using Fixed and write_struct.
pub fn write_fixed_default<A: Fixed + Pod, R: Write + Seek>(val: &A, mut writer: R) -> Result<()> {
    A::move_to_start(&mut writer)?;
    write_struct(val, writer)
}
macro_rules! derive_writable_fixed_via_default {
    ( $t:ty ) => {
        impl crate::bin::WritableFixed for $t {
            fn write_fixed<W: std::io::Write + std::io::Seek>(&self, writer: W) -> std::io::Result<()> {
                crate::bin::write_fixed_default(self, writer)
            }
        }
    };
}

/// A struct that can be written to a bytestream.
pub trait Writable: VarSize {

    /// Write a struct to the output.
    /// This should write exatly [`VarSize::size()`] bytes.
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

/// Concat bytes into a u32 by casting and bitshifting the components.
pub const fn concat_bytes([a, b, c, d]: [u8; 4]) -> u32 {
    (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

/// A struct with a fixed position inside a bytestream where the position is only known at runtime.
/// It provides a way to repeatedly write the struct at the same position.
pub struct Positioned<A> {
    pub position: u64,
    pub data: A,
}
impl<A: Writable> Positioned<A> {
    
    /// Create a new Positioned struct at the current position of the output.
    /// This writes the struct at the current position.
    pub fn new<W: Write + Seek>(data: A, mut out: W) -> Result<Self> {
        let position = out.stream_position()?;
        data.write(&mut out)?;
        Ok(Self { position, data })
    }

    /// Create a new empty Positioned struct at the current position of the output.
    /// This writes the struct at the current position.
    pub fn new_empty<W: Write + Seek>(out: W) ->  Result<Self> 
    where A: Default {
        Self::new(A::default(), out)
    }

    /// Write the current value of this Positioned to the output at its position.
    /// After the update the pointer is returned to it's previous position in the output.
    pub fn update<W: Write + Seek>(&self, mut out: W) -> Result<()> {
        let tmp_pos = out.stream_position()?;
        out.seek(SeekFrom::Start(self.position))?;
        self.data.write(&mut out)?;
        out.seek(SeekFrom::Start(tmp_pos))?;
        Ok(())
    }
}

/// The DataSource provides a way to retrieve a reader.
/// Each reader returned by [`open()`] should be fresh.
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
        A::read_bin(&mut input)
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
