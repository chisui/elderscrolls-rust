use std::{
    io::{BufReader, Read, Write, Seek, SeekFrom, Result, copy, Error, ErrorKind},
    mem::size_of,
    path::Path,
    fs::File,
    fmt,
};
use bytemuck::{Pod, Zeroable};
use thiserror::Error;

use crate::{
    bin::{read_struct, Readable, Writable, DataSource, Positioned},
    Hash,
    Version,
    read::{self, BsaFile},
    write::{self, BsaDirSource},
    magicnumber::MagicNumber,
    str::ZString,
};
use crate::{derive_readable_via_pod, derive_writable_via_pod};


#[derive(Debug, Error)]
#[error("v001 does not support compression")]
pub struct CompressionNotSupported;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Header {
    pub offset_hash_table: u32,
    pub file_count: u32,
}
impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "file_count: {}", self.file_count)
    }
}

impl Readable for Header {
    fn offset(_: &()) -> Option<usize> {
        Some(size_of::<MagicNumber>())
    }
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}
derive_writable_via_pod!(Header);


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FileRecord {
    pub size: u32,
    pub offset: u32,
}
derive_readable_via_pod!(FileRecord);
derive_writable_via_pod!(FileRecord);


const fn offset_after_header() -> u64 {
    (size_of::<MagicNumber>() + size_of::<Header>()) as u64
}
const fn offset_names_start(file_count: u64) -> u64 {
    offset_after_header() + (file_count * (size_of::<FileRecord>() + size_of::<u32>()) as u64)
}
const fn offset_after_index(header: &Header) -> u64 {
    offset_after_header() + header.offset_hash_table as u64 + (size_of::<Hash>() * header.file_count as usize) as u64
}

pub enum V001 {}
pub struct BsaReader<R> {
    pub reader: R,
    pub header: Header,
    pub files: Option<Vec<BsaFile>>,
}
impl<R: Read + Seek> BsaReader<R> {
    fn files(&mut self) -> Result<Vec<BsaFile>> {
        let file_count = self.header.file_count as usize;
        self.reader.seek(SeekFrom::Start(offset_after_header()))?;
        
        let recs = FileRecord::read_many0(&mut self.reader, file_count)?;
        let name_offsets = u32::read_many0(&mut self.reader, file_count)?;
        
        self.reader.seek(SeekFrom::Start(offset_after_header() + self.header.offset_hash_table as u64))?;
        let hashes = Hash::read_many0(&mut self.reader, file_count)?;
        
        recs.iter().zip(name_offsets).zip(hashes)
            .map(|((rec, name_offset), hash)| {
                let name_pos = offset_names_start(file_count as u64) + name_offset as u64;
                self.reader.seek(SeekFrom::Start(name_pos))?;
                let name = match ZString::read_here0(&mut self.reader) {
                    Ok(n) => n,
                    Err(err) => panic!("could not read name at {}: {}", name_pos, err),
                };

                Ok(BsaFile {
                    hash,
                    name: Some(name.to_string()),
                    compressed: false,
                    size: rec.size as usize,
                    offset: offset_after_index(&self.header) + rec.offset as u64,
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
        files: None,
    })
}
impl<R> read::BsaReader for BsaReader<R>
where R: Read + Seek {
    type Header = Header;
    type Root = Vec<BsaFile>;
    
    fn header(&self) -> Header { self.header }
    fn list(&mut self) -> Result<Vec<BsaFile>> {
        if let Some(files) = &self.files {
            Ok(files.to_vec())
        } else {
            let files = self.files()?;
            self.files = Some(files.to_vec());
            Ok(files)
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
        let header = Header {
            offset_hash_table,
            file_count: files.len() as u32,
        };
        header.write_here(&mut out)?;
      
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
        let offset_names_start = offset_names_start(files.len() as u64) as u32;
        for (name_offset, file) in name_offsets.iter_mut().zip(&files) {
            name_offset.data = out.stream_position()? as u32 - offset_names_start;
            println!("write name at {}", name_offset.data);
            let name = ZString::new(&file.name)?;
            name.write_here(&mut out)?;
            name_offset.update(&mut out)?;
        }
        for file in &files {
            Hash::v001(&file.name).write_here(&mut out)?;
        }
        for (rec, file) in recs.iter_mut().zip(&files) {
            let pos = out.stream_position()? as u32;
            println!("write file data at: {}", pos);
            rec.data.offset = pos - offset_after_index(&header) as u32;
            let mut data = file.data.open()?;
            rec.data.size = copy(&mut data, &mut out)? as u32;
            rec.update(&mut out)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::{
        Hash,
        bin::Readable,
        read::BsaReader,
        write::test as w_test,
        Version,
        v001,
    };
    use super::*;

    #[test]
    fn writes_version() {
        let mut bytes = w_test::bsa_bytes::<V001, _>(w_test::some_bsa_dirs());

        let v = Version::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V001);
    }

    #[test]
    fn writes_header() {
        let mut bytes = w_test::bsa_bytes::<V001, _>(w_test::some_bsa_dirs());

        let header = Header::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset_hash_table, 16, "offset_hash_table");
        assert_eq!(header.file_count, 1, "file_count");
    }

    #[test]
    fn write_read_identity_bsa() {
        let dirs = w_test::some_bsa_dirs();
        let bytes = w_test::bsa_bytes::<V001, _>(dirs.clone());
        let mut bsa = v001::read(bytes)
            .unwrap_or_else(|err| panic!("could not open bsa {}", err));
        let files = bsa.list()
            .unwrap_or_else(|err| panic!("could not read dirs {}", err));

    
        assert_eq!(files.len(), 1, "files.len()");
        assert_eq!(files[0].hash, Hash::v001("a\\b"), "files[0].hash");
        assert_eq!(files[0].name, Some("a\\b".to_owned()), "files[0].name");

        let mut data = Vec::<u8>::new();
        bsa.extract(&files[0], &mut data)
            .unwrap_or_else(|err| panic!("could not extract data {}", err));
        assert_eq!(dirs[0].files[0].data, data, "file data");
    }
}
