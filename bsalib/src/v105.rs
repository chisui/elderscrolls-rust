use std::{
    io::{BufReader, Read, Write, Seek, Result, copy},
    path::Path,
    fs::File,
};
use bytemuck::{Zeroable, Pod};


use super::{
    bin::{read_struct, write_struct, Readable, Writable},
    version::Version10X,
    hash::Hash,
    v10x::{self, V10XReader, V10XWriter, V10XWriterOptions, Versioned},
};
pub use super::v104::{ArchiveFlag, Header};


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawDirRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}
impl Readable for RawDirRecord {
    fn read_here<R: Read + Seek>(reader: R, _: &()) -> Result<Self> {
        read_struct(reader)
    }
}
impl Writable for RawDirRecord {
    fn size(&self) -> usize { core::mem::size_of::<Self>() }
    fn write_here<W: Write>(&self, out: W) -> Result<()> {
        write_struct(self, out)
    }
}
impl From<RawDirRecord> for v10x::DirRecord {
    fn from(rec: RawDirRecord) -> Self {
        Self {
            name_hash: rec.name_hash,
            file_count: rec.file_count,
            offset: rec.offset,
        }
    }
}
impl From<v10x::DirRecord> for RawDirRecord {
    fn from(rec: v10x::DirRecord) -> Self {
        Self {
            name_hash: rec.name_hash,
            file_count: rec.file_count,
            _padding_pre: 0,
            offset: rec.offset,
            _padding_post: 0,
        }
    }
}




pub type BsaReader<R> = V10XReader<R, V105, ArchiveFlag, RawDirRecord>;
pub type BsaWriter = V10XWriter<V105, ArchiveFlag, RawDirRecord>;
pub type BsaWriterOptions = V10XWriterOptions<ArchiveFlag>;

pub enum V105 {}

pub fn open<P>(path: P) -> Result<BsaReader<BufReader<File>>>
where P: AsRef<Path> {
    let file = File::open(path)?;
    let buf = BufReader::new(file);
    read(buf)
}
pub fn read<R>(reader: R) -> Result<BsaReader<R>>
where R: Read + Seek {
    BsaReader::read(reader)
}

impl Versioned for V105 {
    fn version() -> Version10X { Version10X::V105 }
 
    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = lz4::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = lz4::EncoderBuilder::new()
            .auto_flush(true)
            .build(&mut writer)?;
        copy(&mut reader, &mut encoder)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, SeekFrom};
    use enumflags2::BitFlags;
    use super::*;
    use crate::{
        str::BZString,
        Hash,
        bin::DataSource,
        read::{BsaReader},
        write::{BsaWriter, BsaDirSource, BsaFileSource},
        version::{Version, Version10X},
        v105,
    };

    #[test]
    fn writes_version() {
        let mut bytes = bsa_bytes(some_bsa_dirs());

        let v = Version::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V10X(Version10X::V105));
    }

    #[test]
    fn writes_header() {
        let mut bytes = bsa_bytes(some_bsa_dirs());

        let header = v105::Header::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset, 36, "offset");
        assert_eq!(header.archive_flags, BitFlags::empty()
            | v105::ArchiveFlag::IncludeFileNames
            | v105::ArchiveFlag::IncludeDirectoryNames);
        assert_eq!(header.dir_count, 1, "dir_count");
        assert_eq!(header.file_count, 1, "file_count");
        assert_eq!(header.total_dir_name_length, 2, "total_dir_name_length");
        assert_eq!(header.total_file_name_length, 2, "total_file_name_length");
        assert_eq!(header.file_flags, BitFlags::empty(), "file_flags");
    }

    #[test]
    fn writes_dir_records() {
        let mut bytes = bsa_bytes(some_bsa_dirs());

        v105::Header::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));
            
        let dirs = RawDirRecord::read_many0(&mut bytes, 1)
            .unwrap_or_else(|err| panic!("could not read dir records {}", err));

        assert_eq!(dirs.len(), 1, "dirs.len()");
        assert_eq!(dirs[0].file_count, 1, "dirs[0].file_count");
        assert_eq!(dirs[0].name_hash, Hash::v10x("a"), "dirs[0].name_hash");
    }

    #[test]
    fn writes_dir_content_records() {
        let mut bytes = bsa_bytes(some_bsa_dirs());


        let header = v105::Header::read0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read Header {}", err));
            
        let dir_rec = v105::RawDirRecord::read_here0(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read dir rec {}", err));
        
        let offset = dir_rec.offset as u64 - header.total_dir_name_length as u64;
        bytes.seek(SeekFrom::Start(offset))
            .unwrap_or_else(|err| panic!("could not seek to offset {}", err));

        let dir_content = v10x::DirContentRecord::read_here(&mut bytes, &(true, 1))
            .unwrap_or_else(|err| panic!("could not read dir content record {}", err));

        assert_eq!(dir_content.name, Some(BZString::new("a").unwrap()), "dir_content.name");
        assert_eq!(dir_content.files.len(), 1, "dir_content.files");
        assert_eq!(dir_content.files[0].name_hash, Hash::v10x("b"), "dir_content.files[0].name_hash");
        assert_eq!(dir_content.files[0].size, 4, "dir_content.files[0].size");
    }

    #[test]
    fn write_read_identity_bsa() {
        check_write_read_identity_bsa(some_bsa_dirs())
    }

    #[test]
    fn write_read_identity_bsa_compressed() {
        let mut dirs = some_bsa_dirs();
        dirs[0].files[0].compressed = Some(true);
        check_write_read_identity_bsa(dirs)
    }

    #[test]
    fn write_read_identity_v105_compession() {
        let mut out = Cursor::new(Vec::<u8>::new());
        let expected: Vec<u8> = vec![1,2,3,4];
      
        v105::V105::compress(Cursor::new(expected.clone()), &mut out)
            .unwrap_or_else(|err| panic!("could not compress data {}", err));
            
        let mut input = Cursor::new(out.into_inner());
        let mut actual = Vec::new();
        v105::V105::uncompress(&mut input,&mut actual)
            .unwrap_or_else(|err| panic!("could not uncompress data {}", err));

        assert_eq!(expected, actual, "compressed data");
    }

    fn check_write_read_identity_bsa(dirs: Vec<BsaDirSource<Vec<u8>>>) {
        let bytes = bsa_bytes(dirs.clone());
        let mut bsa = v105::BsaReader::read(bytes)
            .unwrap_or_else(|err| panic!("could not open bsa {}", err));
        let in_dirs = bsa.dirs()
            .unwrap_or_else(|err| panic!("could not read dirs {}", err));


        assert_eq!(in_dirs.len(), 1, "in_dirs.len()");
        assert_eq!(in_dirs[0].files.len(), 1, "in_dirs[0].files.len()");
        assert_eq!(in_dirs[0].hash, Hash::v10x("a"), "in_dirs[0].name");
        assert_eq!(in_dirs[0].name, Some("a".to_owned()), "in_dirs[0].name");
        assert_eq!(in_dirs[0].files[0].hash, Hash::v10x("b"), "in_dirs[0].files[0].name");
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
        v105::BsaWriter::write_bsa(BsaWriterOptions::default(), dirs, &mut out)
            .unwrap_or_else(|err| panic!("could not write bsa {}", err));
        Cursor::new(out.into_inner())
    }
}
