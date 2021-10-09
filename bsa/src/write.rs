use std::{fs, io::{self, Write, Seek}, path::{Path, PathBuf}, slice::Iter};
use super::bin::DataSource;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsaDirSource<D> {
    pub name: String,
    pub files: Vec<BsaFileSource<D>>,
}
impl<D> BsaDirSource<D> {
    pub fn new<N: Into<String>, I: IntoIterator<Item = BsaFileSource<D>>>(name: N, files: I) -> Self {
        Self {
            name: name.into(),
            files: files.into_iter().collect()
        }
    }
}
impl<'a, D> IntoIterator for &'a BsaDirSource<D> {
    type Item = &'a BsaFileSource<D>;
    type IntoIter = Iter<'a, BsaFileSource<D>>;
    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BsaFileSource<D> {
    pub name: String,
    pub compressed: Option<bool>,
    pub data: D,
}
impl<D> BsaFileSource<D> {
    pub fn new<N: Into<String>>(name: N, data: D) -> Self {
        Self {
            name: name.into(),
            compressed: None,
            data
        }
    }
}
pub trait BsaWriter {
    type Err = io::Error;

    fn write_bsa<DS, D, W>(&self, dirs: DS, out: W) -> Result<(), Self::Err>
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
            BsaDirSource::new("a", [
                BsaFileSource::new("b", vec![1,2,3,4])
            ])
        ]
    }

    pub fn bsa_bytes<W: BsaWriter, D: DataSource>(writer: W, dirs: Vec<BsaDirSource<D>>) -> Cursor<Vec<u8>>
    where
        W::Err: Display,
    {
        let mut out = Cursor::new(Vec::<u8>::new());
        writer.write_bsa(dirs, &mut out)
            .unwrap_or_else(|err| panic!("could not write bsa {}", err));
        Cursor::new(out.into_inner())
    }

    pub fn some_bsa_bytes<W: BsaWriter>() -> Cursor<Vec<u8>>
    where
        W: Default,
        W::Err: Display,
    {
        bsa_bytes(W::default(), some_bsa_dirs())
    }
}
