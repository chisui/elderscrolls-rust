use std::{
    fs,
    path::{Path, PathBuf},
    io::{Result, Write, Seek},
};
use super::bin::DataSource;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsaDirSource<D> {
    pub name: String,
    pub files: Vec<BsaFileSource<D>>,
}
impl<D> BsaDirSource<D> {
    pub fn new(name: String, files: Vec<BsaFileSource<D>>) -> Self {
        Self { name, files }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsaFileSource<D> {
    pub name: String,
    pub compressed: Option<bool>,
    pub data: D,
}
impl<D> BsaFileSource<D> {
    pub fn new(name: String, data: D) -> Self {
        Self { name, compressed: None, data}
    }
}
pub trait BsaWriter {
    type Options;
    fn write_bsa<DS, D, W>(opts: Self::Options, dirs: DS, out: W) -> Result<()>
    where
        D: DataSource,
        DS: IntoIterator<Item = BsaDirSource<D>>,
        W: Write + Seek;
}

pub fn list_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<BsaDirSource<PathBuf>>> {
    let mut stack = vec![PathBuf::new()];
    let mut res = vec![];
    while let Some(path) = stack.pop() {
        let mut files = vec![];
        let cwd = dir.as_ref().join(&path);
        for e in fs::read_dir(cwd)? {
            let entry = e?;
            if entry.file_type()?.is_dir() {
                stack.push([&path, &PathBuf::from(entry.file_name())].iter().collect());
            } else {
                files.push(BsaFileSource {
                    name: entry.file_name().into_string().unwrap(),
                    compressed: None,
                    data: entry.path(),
                });
            }
        }
        if !files.is_empty() {
            res.push(BsaDirSource {
                name: path.into_os_string().into_string().unwrap(), 
                files
            });
        }
    }
    Ok(res)
}
