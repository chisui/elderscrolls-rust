use std::{
    io::{BufReader, Read, Write, Seek, SeekFrom, Result, copy, Error, ErrorKind},
    mem::size_of,
    path::Path,
    fs::File,
};
use bytemuck::{Pod, Zeroable};
use thiserror::Error;

use crate::{
    bin::{read_struct, write_struct, Readable, Writable, DataSource, Positioned},
    Hash,
    Version,
    read::{self, BsaDir, BsaFile},
    write::{self, BsaDirSource},
    magicnumber::MagicNumber,
    str::ZString,
};


#[derive(Debug, Error)]
#[error("v001 does not support compression")]
pub struct CompressionNotSupported;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Header {
    pub offset_hash_table: u32,
    pub files_len: u32,
}
impl Readable for Header {
    fn offset(_: &()) -> Option<usize> {
        Some(size_of::<MagicNumber>())
    }
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}
impl Writable for Header {
    fn size(&self) -> usize {
        size_of::<Self>()
    }
    fn write_here<W: Write>(&self, out: W) -> Result<()> {
        write_struct(self, out)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FileRecord {
    pub size: u32,
    pub offset: u32,
}
impl Readable for FileRecord {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}
impl Writable for FileRecord {
    fn size(&self) -> usize { size_of::<FileRecord>() }
    fn write_here<W: Write>(&self, out: W) -> Result<()> {
        write_struct(self, out)
    }
}

pub enum V001 {}
pub struct BsaReader<R> {
    pub reader: R,
    pub header: Header,
    pub dirs: Option<Vec<BsaDir>>,
}
impl<R: Read + Seek> BsaReader<R> {
    fn files(&mut self) -> Result<Vec<BsaFile>> {
        let files_len = self.header.files_len as usize;
        let offset_after_header = (size_of::<MagicNumber>() + size_of::<Header>()) as u64;
        self.reader.seek(SeekFrom::Start(offset_after_header))?;
        
        let recs = FileRecord::read_many0(&mut self.reader, files_len)?;
        let name_offsets = u32::read_many0(&mut self.reader, files_len)?;
        
        self.reader.seek(SeekFrom::Start(offset_after_header + self.header.offset_hash_table as u64))?;
        let hashes = Hash::read_many0(&mut self.reader, files_len)?;

        let offset_names_start = offset_after_header + (files_len as u64 * (size_of::<FileRecord>() + size_of::<u32>()) as u64);

        recs.iter().zip(name_offsets).zip(hashes)
            .map(|((rec, name_offset), hash)| {
                self.reader.seek(SeekFrom::Start(offset_names_start + name_offset as u64))?;
                let name = ZString::read_here0(&mut self.reader)?;

                Ok(BsaFile {
                    hash,
                    name: Some(name.to_string()),
                    compressed: false,
                    size: rec.size as usize,
                    offset: rec.offset as u64,
                })
            })
            .collect()
    }
}
pub fn open<P>(path: P) -> Result<BsaReader<BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    read(buf)
}
pub fn read<R>(mut reader: R) -> Result<BsaReader<R>>
where R: Read + Seek {
    let header = Header::read0(&mut reader)?;
    Ok(BsaReader {
        reader,
        header,
        dirs: None,
    })
}
impl<R> read::BsaReader for BsaReader<R>
where R: Read + Seek {
    type Header = Header;
    
    fn header(&self) -> Header { self.header }
    fn dirs(&mut self) -> Result<Vec<BsaDir>> {
        if let Some(dirs) = &self.dirs {
            Ok(dirs.to_vec())
        } else {
            let dirs = vec![BsaDir {
                hash: Hash::from(0),
                name: None,
                files: self.files()?,
            }];

            self.dirs = Some(dirs.to_vec());
            Ok(dirs)
        }
    }
    fn extract<W: Write>(&mut self, file: &BsaFile, mut out: W) -> Result<()> {
        self.reader.seek(SeekFrom::Start(file.offset))?;
        let mut data = (&mut self.reader).take(file.size as u64);
        copy(&mut data, &mut out)?;
        Ok(())
    }
}
impl write::BsaWriter for V001 {
    type Options = ();
    
    fn write_bsa<DS, D, W>(_: (), dirs: DS, mut out: W) -> Result<()>
    where
        DS: IntoIterator<Item = BsaDirSource<D>>,
        D: DataSource,
        W: Write + Seek,
    {
        struct OutFile<D> {
            name: String,
            data: D,
        }

        let mut offset_hash_table: u32 = 0;
        let mut files: Vec<OutFile<D>> = Vec::new();
        for dir in dirs {
            for file in dir.files {
                if file.compressed == Some(true) {
                    return Err(Error::new(ErrorKind::InvalidInput, CompressionNotSupported))
                }
                let name = format!("{}\\{}",
                    dir.name.to_lowercase(),
                    file.name.to_lowercase());
                offset_hash_table += (size_of::<FileRecord>() + size_of::<u32>() + name.len() + 1) as u32;
                files.push(OutFile {
                    name,
                    data: file.data,
                });
            }
        }

        Version::V001.write_here(&mut out)?;
        Header {
            offset_hash_table,
            files_len: files.len() as u32,
        }.write_here(&mut out)?;
      
        let mut recs: Vec<Positioned<FileRecord>> = Vec::new();
        for _ in &files {
            recs.push(Positioned::new(FileRecord {
                offset: 0,
                size: 0,
            }, &mut out)?);
        }

        let mut name_offsets: Vec<Positioned<u32>> = Vec::new();
        for _ in &files {
            name_offsets.push(Positioned::new(0, &mut out)?);
        }
        for (name_offset, file) in name_offsets.iter_mut().zip(&files) {
            name_offset.data = out.stream_position()? as u32;
            let name = ZString::new(&file.name)?;
            name.write_here(&mut out)?;
            name_offset.update(&mut out)?;
        }
        for file in &files {
            Hash::v001(&file.name).write_here(&mut out)?;
        }
        for (rec, file) in recs.iter_mut().zip(&files) {
            rec.data.offset = out.stream_position()? as u32;
            let mut data = file.data.open()?;
            rec.data.size = copy(&mut data, &mut out)? as u32;
            rec.update(&mut out)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::{
        Hash,
        bin::{Readable, DataSource},
        read::{BsaReader},
        write::{BsaWriter, BsaDirSource, BsaFileSource},
        Version,
        v001,
    };

    #[test]
    fn writes_version() {
        let mut bytes = bsa_bytes(some_bsa_dirs());

        let v = Version::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V001);
    }

    #[test]
    fn writes_header() {
        let mut bytes = bsa_bytes(some_bsa_dirs());

        let header = v001::Header::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset_hash_table, 16, "offset_hash_table");
        assert_eq!(header.files_len, 1, "files_len");
    }

    #[test]
    fn write_read_identity_bsa() {
        check_write_read_identity_bsa(some_bsa_dirs())
    }


    fn check_write_read_identity_bsa(dirs: Vec<BsaDirSource<Vec<u8>>>) {
        let bytes = bsa_bytes(dirs.clone());
        let mut bsa = v001::read(bytes)
            .unwrap_or_else(|err| panic!("could not open bsa {}", err));
        let in_dirs = bsa.dirs()
            .unwrap_or_else(|err| panic!("could not read dirs {}", err));


        assert_eq!(in_dirs.len(), 1, "in_dirs.len()");
        assert_eq!(in_dirs[0].files.len(), 1, "in_dirs[0].files.len()");
        assert_eq!(in_dirs[0].hash, Hash::v001("a"), "in_dirs[0].name");
        assert_eq!(in_dirs[0].name, Some("a".to_owned()), "in_dirs[0].name");
        assert_eq!(in_dirs[0].files[0].hash, Hash::v001("b"), "in_dirs[0].files[0].name");
        assert_eq!(in_dirs[0].files[0].name, Some("b".to_owned()), "in_dirs[0].files[0].name");

        let mut data = Vec::<u8>::new();
        bsa.extract(&in_dirs[0].files[0], &mut data)
            .unwrap_or_else(|err| panic!("could not extract data {}", err));
        assert_eq!(dirs[0].files[0].data, data, "file data");
    }

    fn some_bsa_dirs() -> Vec<BsaDirSource<Vec<u8>>> {
        vec![
            BsaDirSource::new("a".to_owned(), vec![
                    BsaFileSource::new("b".to_owned(), vec![1,2,3,4])
            ])
        ]
    }

    fn bsa_bytes<D: DataSource>(dirs: Vec<BsaDirSource<D>>) -> Cursor<Vec<u8>> {
        let mut out = Cursor::new(Vec::<u8>::new());
        v001::V001::write_bsa((), dirs, &mut out)
            .unwrap_or_else(|err| panic!("could not write bsa {}", err));
        Cursor::new(out.into_inner())
    }
}
