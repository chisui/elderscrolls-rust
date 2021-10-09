use std::{
    io::{self, Read, Write, Seek, SeekFrom, copy},
    collections::BTreeMap,
    mem::size_of,
    fmt,
};
use bytemuck::{Pod, Zeroable};
use thiserror::Error;

use crate::{EntryId, Hash, Version, bin::{Fixed, ReadableFixed, WritableFixed}, read};
use crate::bin::{
    Readable, Writable, DataSource, Positioned,
    derive_readable_via_pod, derive_writable_via_pod,
};
use crate::write::{self, BsaDirSource};
use crate::str::{StrError, ZString};
use crate::read::BsaFile;
use crate::version::MagicNumber;


#[derive(Debug, Error)]
pub enum V001WriteError {
    #[error("v001 does not support compression")]
    CompressionNotSupported,
    #[error("v001 requires unique hashes. {0} and {1} have the same hash: {}", Hash::v001(.0))]
    HashCollision(String, String),
    #[error("{0}")]
    IO(#[from] io::Error),
    #[error("{1}: \"{0}\"")]
    StrErr(String, StrError),
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct HeaderV001 {
    pub offset_hash_table: u32,
    pub file_count: u32,
}
impl fmt::Display for HeaderV001 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "file_count: {}", self.file_count)
    }
}
impl Fixed for HeaderV001 {
    fn pos() -> usize { size_of::<MagicNumber>() }
}
derive_readable_fixed_via_default!(HeaderV001);
derive_writable_fixed_via_default!(HeaderV001);


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct FileRecord {
    pub size: u32,
    pub offset: u32,
}
derive_readable_via_pod!(FileRecord);
derive_writable_via_pod!(FileRecord);


const fn offset_after_header() -> u64 {
    size_of::<(MagicNumber, HeaderV001)>() as u64
}
const fn offset_names_start(file_count: u64) -> u64 {
    offset_after_header() + (file_count * size_of::<(FileRecord, u32)>() as u64)
}
const fn offset_after_index(header: &HeaderV001) -> u64 {
    offset_after_header() + header.offset_hash_table as u64 + (size_of::<Hash>() * header.file_count as usize) as u64
}

pub struct V001 {}
pub type BsaWriterV001 = V001;
impl Default for V001 {
    fn default() -> Self {
        Self {}
    }
}
pub struct BsaReaderV001<R> {
    reader: R,
    header: HeaderV001,
    files: Option<Vec<BsaFile>>,
}
impl<R: Read + Seek> BsaReaderV001<R> {
    fn files(&mut self) -> io::Result<Vec<BsaFile>> {
        let file_count = self.header.file_count as usize;
        self.reader.seek(SeekFrom::Start(offset_after_header()))?;
        
        let recs = FileRecord::read_bin_many(&mut self.reader, file_count)?;
        let name_offsets = u32::read_bin_many(&mut self.reader, file_count)?;
        
        self.reader.seek(SeekFrom::Start(offset_after_header() + self.header.offset_hash_table as u64))?;
        let hashes = Hash::read_bin_many(&mut self.reader, file_count)?;
        
        recs.iter().zip(name_offsets).zip(hashes)
            .map(|((rec, name_offset), hash)| {
                let name_pos = offset_names_start(file_count as u64) + name_offset as u64;
                self.reader.seek(SeekFrom::Start(name_pos))?;
                let name = match ZString::read_bin(&mut self.reader) {
                    Ok(n) => n,
                    Err(err) => panic!("could not read name at {}: {}", name_pos, err),
                };

                Ok(BsaFile {
                    id: EntryId {
                        hash,
                        name: Some(name.to_string()),
                    },
                    compressed: false,
                    size: rec.size as usize,
                    offset: offset_after_index(&self.header) + rec.offset as u64,
                })
            })
            .collect()
    }
}
impl<R> read::BsaReader for BsaReaderV001<R>
where R: Read + Seek {
    type Header = HeaderV001;
    type Root = Vec<BsaFile>;
    type In = R;
    
    fn read_bsa(mut reader: R) -> io::Result<Self> {
        let header = HeaderV001::read_fixed(&mut reader)?;
        Ok(Self {
            reader,
            header,
            files: None,
        })
    }

    fn header(&self) -> HeaderV001 { self.header }
    fn list(&mut self) -> io::Result<Vec<BsaFile>> {
        if let Some(files) = &self.files {
            Ok(files.to_vec())
        } else {
            let files = self.files()?;
            self.files = Some(files.to_vec());
            Ok(files)
        }
    }
    fn extract<W: Write>(&mut self, file: &BsaFile, mut out: W) -> io::Result<()> {
        self.reader.seek(SeekFrom::Start(file.offset))?;
        let mut data = (&mut self.reader).take(file.size as u64);
        copy(&mut data, &mut out)?;
        Ok(())
    }
}
impl write::BsaWriter for V001 {
    type Err = V001WriteError;
    
    fn write_bsa<DS, D, W>(&self, dirs: DS, mut out: W) -> Result<(), V001WriteError>
    where
        DS: IntoIterator<Item = BsaDirSource<D>>,
        D: DataSource,
        W: Write + Seek,
    {
        let mut offset_hash_table: u32 = 0;
        let mut files: BTreeMap<Hash, (String, D)> = BTreeMap::new(); // has to be ordered by hash
        for dir in dirs {
            for file in dir.files {
                if file.compressed == Some(true) {
                    return Err(V001WriteError::CompressionNotSupported)
                }
                let name = format!("{}\\{}",
                    dir.name.to_lowercase(),
                    file.name.to_lowercase());
                offset_hash_table += (size_of::<(FileRecord, u32)>() + name.len() + 1) as u32;
                let hash = Hash::v001(&name);
                if let Some((other, _)) = files.get(&hash) {
                    return Err(V001WriteError::HashCollision(name, other.clone()))
                }
                files.insert(hash, (name, file.data));
            }
        }

        Version::V001.write_fixed(&mut out)?;
        let header = HeaderV001 {
            offset_hash_table,
            file_count: files.len() as u32,
        };
        header.write_fixed(&mut out)?;
      
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
        for (name_offset, (_, (name, _))) in name_offsets.iter_mut().zip(&files) {
            name_offset.data = out.stream_position()? as u32 - offset_names_start;
            println!("write name at {}", name_offset.data);
            let name = ZString::new(&name)
                .map_err(|err| V001WriteError::StrErr(name.clone(), err))?;
            name.write(&mut out)?;
            name_offset.update(&mut out)?;
        }
        for (hash, _) in &files {
            hash.write(&mut out)?;
        }
        for (rec, (_, (_, data))) in recs.iter_mut().zip(&files) {
            let pos = out.stream_position()? as u32;
            println!("write file data at: {}", pos);
            rec.data.offset = pos - offset_after_index(&header) as u32;
            let mut data = data.open()?;
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
        read::BsaReader,
        write::test::*,
        Version,
    };
    use super::*;

    #[test]
    fn writes_version() {
        let mut bytes = bsa_bytes(V001::default(),some_bsa_dirs());

        let v = Version::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V001);
    }

    #[test]
    fn writes_header() {
        let mut bytes = bsa_bytes(BsaWriterV001::default(), some_bsa_dirs());

        let header = HeaderV001::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset_hash_table, 16, "offset_hash_table");
        assert_eq!(header.file_count, 1, "file_count");
    }

    #[test]
    fn write_read_identity_bsa() {
        let dirs = some_bsa_dirs();
        let bytes = bsa_bytes(BsaWriterV001::default(), dirs.clone());
        let mut bsa = BsaReaderV001::read_bsa(bytes)
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
