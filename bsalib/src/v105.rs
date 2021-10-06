use std::io::{Read, Write, Seek, Result, copy};
use bytemuck::{Zeroable, Pod};

use crate::{
    version::Version10X,
    hash::Hash,
    v10x::{self, BsaReaderV10X, BsaWriterV10X, BsaWriterOptionsV10X, Versioned},
    write,
};
use crate::v104::HeaderV104;
pub use crate::v104::{ArchiveFlag, };


#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct RawDirRecord {
    pub name_hash: Hash,
    pub file_count: u32,
    pub _padding_pre: u32,
    pub offset: u32,
    pub _padding_post: u32,
}
derive_readable_via_pod!(RawDirRecord);
derive_writable_via_pod!(RawDirRecord);
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




pub type HeaderV105 = HeaderV104;
pub type BsaReaderV105<R> = BsaReaderV10X<R, V105, ArchiveFlag, RawDirRecord>;
pub type BsaWriterV105 = BsaWriterV10X<V105, ArchiveFlag, RawDirRecord>;
pub type BsaWriterOptionsV105 = BsaWriterOptionsV10X<ArchiveFlag>;

pub enum V105 {}
impl write::BsaWriter for V105 {
    type Options = <BsaWriterV105 as write::BsaWriter>::Options;

    fn write_bsa<DS, D, W>(opts: Self::Options, dirs: DS, out: W) -> Result<()>
    where
        D: crate::bin::DataSource,
        DS: IntoIterator<Item = write::BsaDirSource<D>>,
        W: Write + Seek {
        BsaWriterV105::write_bsa(opts, dirs, out)
    }
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
    use crate::{Hash, bin::{Readable, ReadableFixed, ReadableParam}, read::{BsaReader}, str::BZString, v105, version::{Version, Version10X}, write::{BsaDirSource, test::*}};

    #[test]
    fn writes_version() {
        let mut bytes = some_bsa_bytes::<V105>();

        let v = Version::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V10X(Version10X::V105));
    }

    #[test]
    fn writes_header() {
        let mut bytes = some_bsa_bytes::<V105>();

        let header = HeaderV105::read_fixed(&mut bytes)
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
        let mut bytes = some_bsa_bytes::<V105>();

        HeaderV105::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));
            
        let dirs = RawDirRecord::read_bin_many(&mut bytes, 1)
            .unwrap_or_else(|err| panic!("could not read dir records {}", err));

        assert_eq!(dirs.len(), 1, "dirs.len()");
        assert_eq!(dirs[0].file_count, 1, "dirs[0].file_count");
        assert_eq!(dirs[0].name_hash, Hash::v10x("a"), "dirs[0].name_hash");
    }

    #[test]
    fn writes_dir_content_records() {
        let mut bytes = some_bsa_bytes::<V105>();


        let header = HeaderV105::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read Header {}", err));
            
        let dir_rec = v105::RawDirRecord::read_bin(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read dir rec {}", err));
        
        let offset = dir_rec.offset as u64 - header.total_dir_name_length as u64;
        bytes.seek(SeekFrom::Start(offset))
            .unwrap_or_else(|err| panic!("could not seek to offset {}", err));

        let dir_content = v10x::DirContentRecord::read_with_param(&mut bytes, (true, 1))
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
        let bytes = bsa_bytes::<V105, _>(dirs.clone());
        let mut bsa = BsaReaderV105::read_bsa(bytes)
            .unwrap_or_else(|err| panic!("could not open bsa {}", err));
        let in_dirs = bsa.list()
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
}
