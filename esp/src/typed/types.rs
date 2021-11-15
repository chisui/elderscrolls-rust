use std::io::{self, Read};
use std::fmt;
use std::str;

use thiserror::Error;
use bytemuck::{Pod, PodCastError, Zeroable};

use crate::bin::{Readable, ReadStructExt};
use crate::raw;


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct BlockId(u32);
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}
impl From<raw::Label> for BlockId {
    fn from(l: raw::Label) -> Self {
        Self(u32::from_le_bytes(l.0))
    }
}
impl<R: Read> Readable<R> for BlockId {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point2D<C> {
    pub x: C,
    pub y: C,
}
impl From<raw::Label> for Point2D<u16> {
    fn from(l: raw::Label) -> Self {
        Self {
            x: u16::from_le_bytes([
                l.0[0],
                l.0[1],
            ]),
            y: u16::from_le_bytes([
                l.0[2],
                l.0[3],
            ]),
        }
    }
}
impl<C, R> Readable<R> for Point2D<C>
where C: Readable<R> {
    type Error = C::Error;
    fn read_val(reader: &mut R) -> Result<Self, C::Error> {
        Ok(Self {
            x: C::read_val(reader)?,
            y: C::read_val(reader)?,
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct FormId(u32);
impl fmt::Display for FormId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}
impl From<raw::Label> for FormId {
    fn from(l: raw::Label) -> Self {
        Self(u32::from_le_bytes(l.0))
    }
}
impl<R: Read> Readable<R> for FormId {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Zeroable, Pod)]
pub struct ObjectId(u32);

impl<R: Read> Readable<R> for ObjectId {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypedValue {
    Bool(bool),
    Int(u32),
    Float(f32),
    LString(LString),
    Other(Vec<u8>),
}
impl TypedValue {
    pub fn new(c: char, data: Vec<u8>) -> Result<Self, PodCastError> {
        match c {
            'b' => {
                let v = bytemuck::try_from_bytes::<u32>(data.as_ref())?;
                Ok(TypedValue::Bool(*v == 0))
            },
            'i' => {
                bytemuck::try_from_bytes::<u32>(data.as_ref())
                    .map(|i| TypedValue::Int(*i))
            },
            'f' => {
                bytemuck::try_from_bytes::<f32>(data.as_ref())
                    .map(|f| TypedValue::Float(*f))
            },
            's' | 'S' => {
                bytemuck::try_from_bytes::<LString>(data.as_ref())
                    .map(|s| TypedValue::LString(*s))
            },
            _ => Ok(TypedValue::Other(data)),
        }
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct LString(pub u32);
impl<R: Read> Readable<R> for LString {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl<R: Read> Readable<R> for Color {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EditorID(pub String);
impl<R: Read> Readable<R> for EditorID {
    type Error = StringError;
    fn read_val(reader: &mut R) -> Result<Self, StringError> {
        let ZString(s) = ZString::read_val(reader)?;
        Ok(Self(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(String);
impl Path {
    pub fn new(s: String) -> Self {
        Self(s.replace("\\", "/"))
    }
}
impl<R: Read> Readable<R> for Path {
    type Error = StringError;
    fn read_val(reader: &mut R) -> Result<Self, StringError> {
        let ZString(s) = ZString::read_val(reader)?;
        Ok(Self::new(s))
    }
}


#[derive(Debug, Error)]
pub enum StringError {
    #[error("{0}")] IO(#[from] io::Error),
    #[error("{0}")] Utf8(#[from] str::Utf8Error),
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZString(pub String);
impl ToString for ZString {
    fn to_string(&self) -> String {
        self.0.to_owned()
    }
}
impl From<ZString> for String {
    fn from(val: ZString) -> Self {
        val.0
    }
}
impl From<&ZString> for String {
    fn from(val: &ZString) -> Self {
        val.0.to_owned()
    }
}
impl<R: Read> Readable<R> for ZString {
    type Error = StringError;
    fn read_val(reader: &mut R) -> Result<Self, StringError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let s = str::from_utf8(&bytes[0 .. bytes.len() - 1])?;
        Ok(Self(s.to_owned()))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct ObjectBounds {
    low: Point3D<u16>,
    high: Point3D<u16>,
}
impl<R: Read> Readable<R> for ObjectBounds {
    fn read_val(reader: &mut R) -> io::Result<Self> {
        reader.read_struct()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point3D<C> {
    pub x: C,
    pub y: C,
    pub z: C,
}
unsafe impl<C> Zeroable for Point3D<C>
where C: Zeroable {
    fn zeroed() -> Self {
        Self {
            x: C::zeroed(),
            y: C::zeroed(),
            z: C::zeroed(),
        }
    }
}
unsafe impl<C: Pod> Pod for Point3D<C> {}

impl<C, R> Readable<R> for Point3D<C>
where C: Readable<R> {
    type Error = C::Error;
    fn read_val(reader: &mut R) -> Result<Self, C::Error> {
        Ok(Self {
            x: C::read_val(reader)?,
            y: C::read_val(reader)?,
            z: C::read_val(reader)?,
        })
    }
}

pub type ActorValue = u8;
