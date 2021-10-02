use std::{
    io::{BufReader, Result, Error, ErrorKind},
    fs::{self, File},
    path::PathBuf,
    fmt,
};
use clap::Clap;
use glob::{Pattern, MatchOptions};
use thiserror::Error;

use bsalib::{
    SomeBsaReader,
    SomeBsaHeader,
    archive::{self, BsaReader, BsaWriter, FileId},
    v105,
    v10x::{ToArchiveBitFlags, V10XHeader},
};
mod cli;
use crate::cli::{Cmds, Info, List, Extract, Create};


fn main() -> Result<()> {
    Cmds::parse().exec()
}

trait Cmd {
    fn exec(&self) -> Result<()>;
}
impl Cmd for Cmds {
    fn exec(&self) -> Result<()> {
        match self {
            Cmds::Info(cmd)    => cmd.exec(),
            Cmds::List(cmd)    => cmd.exec(),
            Cmds::Extract(cmd) => cmd.exec(),
            Cmds::Create(cmd)  => cmd.exec(),   
        }
    }
}

impl Cmd for Info {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;
        let bsa = SomeBsaReader::open(&mut reader)?;
        if self.verbose {
            println!("{}", bsa.header());
        } else {
            match bsa.header() {
                SomeBsaHeader::V103(h) => println!("{}", Sparse(h)),
                SomeBsaHeader::V104(h) => println!("{}", Sparse(h)),
                SomeBsaHeader::V105(h) => println!("{}", Sparse(h)),
            }
        }
        Ok(())
    }
}

impl Cmd for List {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let mut bsa = SomeBsaReader::open(&mut reader)?;
        for dir in bsa.read_dirs()? {
            for file in dir.files {
                if self.attributes {
                    let c = if file.compressed { "c" } else { " " };
                    println!("{0} {1: >8} {2}/{3}", c, file.size / 1000, dir.name, file.name);
                } else {
                    println!("{0}/{1}", dir.name, file.name);
                }
            }
        }
        Ok(())
    }
}


struct FileMatcher {
    patterns: Vec<Pattern>,
}
impl FileMatcher {
    fn new(patterns: &Vec<Pattern>) -> Self {
        FileMatcher {
            patterns: patterns.clone()
        }
    }

    fn matches(&self, path: &String) -> bool {
        let match_opt = MatchOptions {
            case_sensitive: false,
            require_literal_leading_dot: false,
            require_literal_separator: false,
        };
        self.patterns.is_empty()
            || self.patterns.iter().any(|p|
                p.matches_with(&path, match_opt)
                || path.starts_with(p.as_str()))
    }
}

impl Cmd for Extract {
    fn exec(&self) -> Result<()> {
        let matcher = FileMatcher::new(&self.paths);

        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let mut bsa = SomeBsaReader::open(&mut reader)?;

        let dirs = bsa.read_dirs()?;
        for dir in dirs {
            for file in dir.files {
                let file_path = format!("{}/{}", dir.name, file.name);
                if matcher.matches(&file_path) {
                    println!("{}", file_path);
                    let mut out = open_output_file(&self.output, &dir.name, &file.name)?;
                    bsa.extract(&file, &mut out)?;
                }
            }
        }

        Ok(())
    }
}

fn open_output_file(out: &PathBuf, dir: &FileId, file: &FileId) -> Result<File> {
    let mut path_buf = PathBuf::from(out);
    path_buf.push(format!("{}", dir));
    fs::create_dir_all(&path_buf)?;
    path_buf.push(format!("{}", file));
    check_exists(&path_buf)?;
    File::create(path_buf.as_path())
}

fn check_exists(path: &PathBuf) -> Result<()> {
    if path.exists() {
        Err(Error::new(ErrorKind::AlreadyExists, FileAlreadyExists(path.clone())))
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("{0} already exists")]
struct FileAlreadyExists(PathBuf);


impl Cmd for Create {
    fn exec(&self) -> Result<()> {
        let output = match self.output.as_ref() {
            Some(p) => p.clone(),
            None => {
                let mut tmp = (&self).file.clone();
                tmp.set_extension("bsa");
                tmp.to_owned()
            },
        };

        check_exists(&output)?;

        let mut opts = v105::BsaWriterOptions::default();
        if self.compress {
            opts.archive_flags |= v105::ArchiveFlag::CompressedArchive;
        }

        if self.embed_file_names {
            opts.archive_flags |= v105::ArchiveFlag::EmbedFileNames;
        }
        
        let dirs = archive::list_dir(&self.file)?;
        let file = File::create(output)?;
        v105::BsaWriter::write_bsa(opts, dirs, file)
    }
}


struct Sparse<A>(A);

impl<AF: ToArchiveBitFlags + fmt::Debug> fmt::Display for Sparse<V10XHeader<AF>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Direcotries: {}", self.0.dir_count)?;
        writeln!(f, "Files:   {}", self.0.file_count)?;
        writeln!(f, "Archive flags:")?;
        for flag in self.0.archive_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        writeln!(f, "File flags:")?;
        for flag in self.0.file_flags.iter() {
            writeln!(f, "    {:?}", flag)?;
        }
        Ok(())
    }
}
