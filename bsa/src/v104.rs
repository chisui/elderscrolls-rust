use std::str;
use enumflags2::{bitflags, BitFlags};

use crate::compress::ZLib;
use crate::v10x::{
    ReaderV10X,
    HeaderV10X,
    WriterV10X,
    DirRecord,
    ToArchiveBitFlags,
    Versioned,
};
use crate::version::Version10X;


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArchiveFlagV104 {
    /// The game may not load a BSA without this bit set.
    IncludeDirectoryNames = 0x1,
    /// The game may not load a BSA without this bit set.
    IncludeFileNames = 0x2,
    /// This does not mean all files are compressed. It means they are
    /// compressed by default.
    CompressedArchive = 0x4,
    RetainDirectoryNames = 0x8,
    /// Unknown, but observed being set in official BSA files containing
    /// sounds (but not voices). Possibly instructs the game to retain
    /// file names in memory.
    RetainFileNames = 0x10,
    RetainFileNameOffsets = 0x20,
    /// Hash values and numbers after the header are encoded big-endian.
    Xbox360Archive = 0x40,
    RetainStringsDuringStartup = 0x80,
    /// Embed File Names. Indicates the file data blocks begin with a
    /// bstring containing the full path of the file. For example, in
    /// \"Skyrim - Textures.bsa\" the first data block is
    /// $2B textures/effects/fxfluidstreamdripatlus.dds
    /// ($2B indicating the name is 43 bytes). The data block begins
    /// immediately after the bstring.
    EmbedFileNames = 0x100,
    /// This can only be used with CompressedArchive.
    /// This is an Xbox 360 only compression algorithm.
    XMemCodec = 0x200,
}


impl ToArchiveBitFlags for ArchiveFlagV104 {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }
    fn from_archive_bit_flags(flags: BitFlags<Self>) -> u32 { 
        flags.bits()
    }

    fn is_compressed_by_default() -> Self { ArchiveFlagV104::CompressedArchive }
    fn includes_file_names() -> Self { ArchiveFlagV104::IncludeFileNames }
    fn includes_dir_names() -> Self { ArchiveFlagV104::IncludeDirectoryNames }
    fn embed_file_names() -> Option<Self> { Some(ArchiveFlagV104::EmbedFileNames) }
}

pub type HeaderV104 = HeaderV10X<ArchiveFlagV104>;
pub type ReaderV104<R> = ReaderV10X<R, V104, ZLib, ArchiveFlagV104, DirRecord>;
pub type WriterV104 = WriterV10X<V104, ZLib, ArchiveFlagV104, DirRecord>;

pub enum V104 {}
impl Versioned for V104 {
    fn version() -> Version10X { Version10X::V104 }
}


#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};
    use enumflags2::BitFlags;
    use super::*;
    use crate::Hash;
    use crate::write::{Dir, test::*};
    use crate::version::{Version, Version10X};
    use crate::v10x;
    use crate::v104;
    use crate::str::BZString;
    use crate::read::Reader;
    use crate::compress::Compression;
    use crate::bin::{Readable, ReadableFixed, ReadableParam};

    #[test]
    fn writes_version() {
        let mut bytes = some_bsa_bytes::<WriterV104>();

        let v = Version::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V10X(Version10X::V104));
    }

    #[test]
    fn writes_header() {
        let mut bytes = some_bsa_bytes::<WriterV104>();

        let header = HeaderV104::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset, 36, "offset");
        assert_eq!(header.archive_flags, BitFlags::empty()
            | v104::ArchiveFlagV104::IncludeFileNames
            | v104::ArchiveFlagV104::IncludeDirectoryNames);
        assert_eq!(header.dir_count, 1, "dir_count");
        assert_eq!(header.file_count, 1, "file_count");
        assert_eq!(header.total_dir_name_length, 2, "total_dir_name_length");
        assert_eq!(header.total_file_name_length, 2, "total_file_name_length");
        assert_eq!(header.file_flags, BitFlags::empty(), "file_flags");
    }

    #[test]
    fn writes_dir_records() {
        let mut bytes = some_bsa_bytes::<WriterV104>();

        HeaderV104::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));
            
        let dirs = DirRecord::read_bin_many(&mut bytes, 1)
            .unwrap_or_else(|err| panic!("could not read dir records {}", err));

        assert_eq!(dirs.len(), 1, "dirs.len()");
        assert_eq!(dirs[0].file_count, 1, "dirs[0].file_count");
        assert_eq!(dirs[0].name_hash, Hash::v10x("a"), "dirs[0].name_hash");
    }

    #[test]
    fn writes_dir_content_records() {
        let mut bytes = some_bsa_bytes::<WriterV104>();


        let header = HeaderV104::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read Header {}", err));
            
        let dir_rec = v104::DirRecord::read_bin(&mut bytes)
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
    fn write_read_identity_v104_compession() {
        let mut out = Cursor::new(Vec::<u8>::new());
        let expected: Vec<u8> = vec![1,2,3,4];
      
        ZLib::compress(Cursor::new(expected.clone()), &mut out)
            .unwrap_or_else(|err| panic!("could not compress data {}", err));
            
        let mut input = Cursor::new(out.into_inner());
        let mut actual = Vec::new();
        ZLib::uncompress(&mut input,&mut actual)
            .unwrap_or_else(|err| panic!("could not uncompress data {}", err));

        assert_eq!(expected, actual, "compressed data");
    }

    fn check_write_read_identity_bsa(dirs: Vec<BsaDirSource<Vec<u8>>>) {
        let bytes = bsa_bytes(WriterV104::default(), dirs.clone());
        let mut bsa = ReaderV104::read_bsa(bytes)
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
