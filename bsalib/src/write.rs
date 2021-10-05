use std::{
    fs,
    path::{Path, PathBuf},
    io::{self, Write, Seek},
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
    type Err = io::Error;

    fn write_bsa<DS, D, W>(opts: Self::Options, dirs: DS, out: W) -> Result<(), Self::Err>
    where
        D: DataSource,
        DS: IntoIterator<Item = BsaDirSource<D>>,
        W: Write + Seek;
}

pub fn list_dir<P: AsRef<Path>>(dir: P) -> io::Result<Vec<BsaDirSource<PathBuf>>> {
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

#[cfg(test)]
pub(crate) mod test {
    use std::{io::Cursor, fmt::Display};

    use super::*;

    pub fn some_bsa_dirs() -> Vec<BsaDirSource<Vec<u8>>> {
        vec![
            BsaDirSource::new("a".to_owned(), vec![
                BsaFileSource::new("b".to_owned(), vec![1,2,3,4])
            ])
        ]
    }

    pub fn bsa_bytes<W: BsaWriter, D: DataSource>(dirs: Vec<BsaDirSource<D>>) -> Cursor<Vec<u8>>
    where
        W::Options: Default,
        W::Err: Display,
    {
        let mut out = Cursor::new(Vec::<u8>::new());
        W::write_bsa(W::Options::default(), dirs, &mut out)
            .unwrap_or_else(|err| panic!("could not write bsa {}", err));
        Cursor::new(out.into_inner())
    }

    pub fn some_bsa_bytes<W: BsaWriter>() -> Cursor<Vec<u8>>
    where
        W::Options: Default,
        W::Err: Display,
    {
        bsa_bytes::<W, _>(some_bsa_dirs())
    }
}
