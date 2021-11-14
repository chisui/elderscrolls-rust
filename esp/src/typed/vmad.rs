
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttachedScriptsData {
    pub scripts: Vec<AttachedScriptData>,
    pub fragments: Vec<Fragment>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttachedScriptData {
    pub name: String,
    pub status: AttachedScriptStatus,
    pub props: Vec<AttachedScriptProperty>,
}
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
pub enum AttachedScriptStatus {
    LocalScript = 0,
    InheritedPropsChanged = 1,
    InheritedRemoved = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AttachedScriptProperty {
    pub name: String,
    pub status: PropertyStatus,
    pub value: PapyrusValueContainer,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive)]
pub enum PropertyStatus {
    Edited = 1,
    Removed = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PapyrusValueContainer {
    Value(PapyrusValue),
    Array(Vec<PapyrusValue>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PapyrusValue {
    Object {
        form: FormId,
        alias: i16,
        unused: u16,
    },
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
}
