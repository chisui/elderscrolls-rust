use std::{
    io::{Read, Write, Seek, Result, copy},
    fmt,
};
use bytemuck::{Zeroable, Pod};


pub use super::{
    bin::{read_struct, write_struct, Readable, Writable},
    archive::Bsa,
    version::{Version, Version10X},
    hash::{hash_v10x, Hash},
    v10x::{self, V10XArchive, V10XWriter, V10XWriterOptions, Versioned, DirContentRecord},
    v104::{ArchiveFlag, Header, BZString},
};


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

pub enum V105T{}
impl Versioned for V105T {
    fn version() -> Version { Version::V10X(Version10X::V105) }
    fn fmt_version(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BSA v105 file, format used by: TES V: Skyrim Special Edition")
    }

    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = lz4::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = lz4::EncoderBuilder::new()
            .build(&mut writer)?;
        copy(&mut reader, &mut encoder)
    }
}

pub type BsaArchive<R> = V10XArchive<R, V105T, ArchiveFlag, RawDirRecord>;
pub type BsaWriter = V10XWriter<V105T, ArchiveFlag, RawDirRecord>;
pub type BsaWriterOptions = V10XWriterOptions<ArchiveFlag>;


#[cfg(test)]
mod tests {
    use std::io::{Cursor, SeekFrom};
    use enumflags2::BitFlags;
    use crate::bin::DataSource;
    use crate::archive::{FileId, BsaWriter, Bsa, BsaDirSource, BsaFileSource};
    use crate::version::{Version, Version10X};
    use crate::v105;
    use super::*;

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
        assert_eq!(dirs[0].name_hash, hash_v10x("a"), "dirs[0].name_hash");
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

        let dir_content = v105::DirContentRecord::read_here(&mut bytes, &(true, 1))
            .unwrap_or_else(|err| panic!("could not read dir content record {}", err));

        assert_eq!(dir_content.name, Some(BZString::new("a").unwrap()), "dir_content.name");
        assert_eq!(dir_content.files.len(), 1, "dir_content.files");
        assert_eq!(dir_content.files[0].name_hash, hash_v10x("b"), "dir_content.files[0].name_hash");
        assert_eq!(dir_content.files[0].size, 4, "dir_content.files[0].size");
    }

    #[test]
    fn write_read_identity() {
        check_write_read_identity(some_bsa_dirs())
    }

    #[test]
    fn write_read_identity_compressed() {
        let mut dirs = some_bsa_dirs();
        dirs[0].files[0].compressed = Some(true);
        check_write_read_identity(dirs)
    }

    fn check_write_read_identity(dirs: Vec<BsaDirSource<Vec<u8>>>) {
        let bytes = bsa_bytes(dirs.clone());
        let mut bsa = v105::BsaArchive::open(bytes)
            .unwrap_or_else(|err| panic!("could not open bsa {}", err));
        let in_dirs = bsa.read_dirs()
            .unwrap_or_else(|err| panic!("could not read dirs {}", err));


        assert_eq!(in_dirs.len(), 1, "in_dirs.len()");
        assert_eq!(in_dirs[0].files.len(), 1, "in_dirs[0].files.len()");
        assert_eq!(in_dirs[0].name, FileId::String("a".to_owned()), "in_dirs[0].name");
        assert_eq!(in_dirs[0].files[0].name, FileId::String("b".to_owned()), "in_dirs[0].files[0].name");

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
