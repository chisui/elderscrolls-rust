use std::{
    io::{BufReader, Read, Write, Seek, SeekFrom, Result, copy},
    mem::size_of,
    path::Path,
    fs::File,
};
use bytemuck::{Pod, Zeroable};

use crate::{
    bin::{read_struct, write_struct, Readable, Writable},
    Hash,
    read::{self, BsaDir, BsaFile},
    magicnumber::MagicNumber,
    str::ZString,
};


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
