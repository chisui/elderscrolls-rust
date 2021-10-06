use std::{
    io::{BufReader, Read, Seek, Write, Result, copy},
    str,
    path::Path,
    fs::File,
};
use enumflags2::{bitflags, BitFlags};
use libflate::zlib;

use crate::{
    version::Version10X,
    v10x::{
        V10XReader,
        V10XHeader,
        V10XWriter,
        V10XWriterOptions,
        ToArchiveBitFlags,
        Versioned,
        DirRecord
    },
    write,
};


#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ArchiveFlag {
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeDirectoryNames = 0x1,
    #[doc = "The game may not load a BSA without this bit set."]
    IncludeFileNames = 0x2,
    #[doc = "This does not mean all files are compressed. It means they are"]
    #[doc = "compressed by default."]
    CompressedArchive = 0x4,
    RetainDirectoryNames = 0x8,
    #[doc = "Unknown, but observed being set in official BSA files containing"]
    #[doc = "sounds (but not voices). Possibly instructs the game to retain"]
    #[doc = "file names in memory."]
    RetainFileNames = 0x10,
    RetainFileNameOffsets = 0x20,
    #[doc = "Hash values and numbers after the header are encoded big-endian."]
    Xbox360Archive = 0x40,
    Ux80  = 0x80,
    Ux100 = 0x100,
    Ux200 = 0x200,
    Ux400 = 0x400,
}
impl ToArchiveBitFlags for ArchiveFlag {
    fn to_archive_bit_flags(bits: u32) -> BitFlags<Self> {
        BitFlags::from_bits_truncate(bits)
    }
    fn from_archive_bit_flags(flags: BitFlags<Self>) -> u32 { 
        flags.bits()
    }
    

    fn is_compressed_by_default() -> Self { ArchiveFlag::CompressedArchive }
    fn includes_file_names() -> Self { ArchiveFlag::IncludeFileNames }
    fn includes_dir_names() -> Self { ArchiveFlag::IncludeDirectoryNames }
}

pub enum V103 {}
impl write::BsaWriter for V103 {
    type Options = <BsaWriter as write::BsaWriter>::Options;

    fn write_bsa<DS, D, W>(opts: Self::Options, dirs: DS, out: W) -> Result<()>
    where
        D: crate::bin::DataSource,
        DS: IntoIterator<Item = write::BsaDirSource<D>>,
        W: Write + Seek {
        BsaWriter::write_bsa(opts, dirs, out)
    }
}

impl Versioned for V103 {
    fn version() -> Version10X { Version10X::V103 }
   
    fn uncompress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut decoder = zlib::Decoder::new(&mut reader)?;
        copy(&mut decoder, &mut writer)
    }

    fn compress<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<u64> {
        let mut encoder = zlib::Encoder::new(&mut writer)?;
        let size = copy(&mut reader, &mut encoder)?;
        encoder.finish().into_result()?;
        Ok(size)
    }
}

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

pub type Header = V10XHeader<ArchiveFlag>;
pub type BsaReader<R> = V10XReader<R, V103, ArchiveFlag, DirRecord>;
pub type BsaWriter = V10XWriter<V103, ArchiveFlag, DirRecord>;
pub type BsaWriterOptions = V10XWriterOptions<ArchiveFlag>;


#[cfg(test)]
mod tests {
    use std::io::{Cursor, SeekFrom};
    use enumflags2::BitFlags;
    use super::*;
    use crate::{Hash, bin::{Readable, ReadableParam, ReadableFixed}, read::BsaReader, str::BZString, v103, v10x, version::{Version, Version10X}, write::{BsaDirSource, test::*}};

    #[test]
    fn writes_version() {
        let mut bytes = some_bsa_bytes::<V103>();

        let v = Version::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read version {}", err));
        assert_eq!(v, Version::V10X(Version10X::V103));
    }

    #[test]
    fn writes_header() {
        let mut bytes = some_bsa_bytes::<V103>();

        let header = v103::Header::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));

        assert_eq!(header.offset, 36, "offset");
        assert_eq!(header.archive_flags, BitFlags::empty()
            | v103::ArchiveFlag::IncludeFileNames
            | v103::ArchiveFlag::IncludeDirectoryNames);
        assert_eq!(header.dir_count, 1, "dir_count");
        assert_eq!(header.file_count, 1, "file_count");
        assert_eq!(header.total_dir_name_length, 2, "total_dir_name_length");
        assert_eq!(header.total_file_name_length, 2, "total_file_name_length");
        assert_eq!(header.file_flags, BitFlags::empty(), "file_flags");
    }

    #[test]
    fn writes_dir_records() {
        let mut bytes = some_bsa_bytes::<V103>();

        v103::Header::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read header {}", err));
            
        let dirs = DirRecord::read_many(&mut bytes, 1)
            .unwrap_or_else(|err| panic!("could not read dir records {}", err));

        assert_eq!(dirs.len(), 1, "dirs.len()");
        assert_eq!(dirs[0].file_count, 1, "dirs[0].file_count");
        assert_eq!(dirs[0].name_hash, Hash::v10x("a"), "dirs[0].name_hash");
    }

    #[test]
    fn writes_dir_content_records() {
        let mut bytes = some_bsa_bytes::<V103>();


        let header = v103::Header::read_fixed(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read Header {}", err));
            
        let dir_rec = v103::DirRecord::read(&mut bytes)
            .unwrap_or_else(|err| panic!("could not read dir rec {}", err));
        
        let offset = dir_rec.offset as u64 - header.total_dir_name_length as u64;
        bytes.seek(SeekFrom::Start(offset))
            .unwrap_or_else(|err| panic!("could not seek to offset {}", err));

        let dir_content = v10x::DirContentRecord::read(&mut bytes, (true, 1))
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
    fn write_read_identity_v103_compession() {
        let mut out = Cursor::new(Vec::<u8>::new());
        let expected: Vec<u8> = vec![1,2,3,4];
      
        v103::V103::compress(Cursor::new(expected.clone()), &mut out)
            .unwrap_or_else(|err| panic!("could not compress data {}", err));
            
        let mut input = Cursor::new(out.into_inner());
        let mut actual = Vec::new();
        v103::V103::uncompress(&mut input,&mut actual)
            .unwrap_or_else(|err| panic!("could not uncompress data {}", err));

        assert_eq!(expected, actual, "compressed data");
    }

    fn check_write_read_identity_bsa(dirs: Vec<BsaDirSource<Vec<u8>>>) {
        let bytes = bsa_bytes::<V103, _>(dirs.clone());
        let mut bsa = v103::BsaReader::read(bytes)
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
