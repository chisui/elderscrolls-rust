use std::fmt;

use bytemuck::{Pod, PodCastError, Zeroable};

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Zeroable, Pod)]
pub struct ObjectId(u32);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypedValue {
    Bool(bool),
    Int(u32),
    Float(f32),
    LString(LString),
    Other(Vec<u8>),
}
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct LString(pub u32);
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
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EditorID(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(String);
impl Path {
    pub fn new(s: String) -> Self {
        Self(s.replace("\\", "/"))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Zeroable, Pod)]
pub struct ObjectBounds {
    low: Point3D<u16>,
    high: Point3D<u16>,
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
unsafe impl<C> Pod for Point3D<C>
where C: Pod {}
