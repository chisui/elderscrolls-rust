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
    self,
    SomeBsaHeader, SomeBsaReader, SomeBsaRoot,
    Version, Version10X,
    read::{BsaReader, BsaEntry, EntryId},
    write::{BsaWriter, list_dir},
    v105,
    v10x::{ToArchiveBitFlags, V10XHeader},
};
mod cli;
use crate::cli::{Cmds, Info, List, Extract, Create, Overrides};


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
        let bsa = open(&self.file, &self.overrides)?;
        if self.verbose {
            println!("{}", bsa.header());
        } else {
            match bsa.header() {
                SomeBsaHeader::V001(h) => println!("{}", h),
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
        let mut bsa = open(&self.file, &self.overrides)?;
        match bsa.dirs()? {
            SomeBsaRoot::Dirs(dirs) => {
                for dir in &dirs {
                    for file in dir {
                        if self.attributes {
                            let c = if file.compressed { "c" } else { " " };
                            println!("{0} {1: >8} {2}/{3}", c, file.size / 1000, dir.id(), file.id());
                        } else {
                            println!("{0}/{1}", dir.id(), file.id());
                        }
                    }
                }
            },
            SomeBsaRoot::Files(files) => {
                for file in &files {
                    if self.attributes {
                        println!("  {0: >8} {1}", file.size / 1000, file.id());
                    } else {
                        println!("{0}", file.id());
                    }
                }
            },
        }
        Ok(())
    }
}


enum FileMatcher {
    Any,
    Include(Vec<Pattern>),
    Exclude(Vec<Pattern>),
}
impl FileMatcher {
    const MATCH_OPTS: MatchOptions = MatchOptions {
        case_sensitive: false,
        require_literal_leading_dot: false,
        require_literal_separator: false,
    };

    fn new(include: &Vec<Pattern>, exclude: &Vec<Pattern>) -> Result<Self> {
        Ok(match (include.is_empty(), exclude.is_empty()) {
            (true, true) => FileMatcher::Any,
            (false, true) => FileMatcher::Include(include.clone()),
            (true, false) => FileMatcher::Exclude(exclude.clone()),
            (false, false) => Err(Error::new(ErrorKind::InvalidInput, "--include can not be used in combination with --exclude"))?,
        })
    }

    fn matches(&self, path: &String) -> bool {
        match self {
            FileMatcher::Any => true,
            FileMatcher::Include(patterns) => patterns.iter()
                .any(|p| FileMatcher::match_single(p, path)),
            FileMatcher::Exclude(patterns) => patterns.iter()
                .all(|p| !FileMatcher::match_single(p, path)),
        }
    }

    fn match_single(pattern: &Pattern, path: &String) -> bool {
        pattern.matches_with(&path, FileMatcher::MATCH_OPTS) || path.starts_with(pattern.as_str())
    }
}

impl Cmd for Extract {
    fn exec(&self) -> Result<()> {
        let matcher = FileMatcher::new(&self.include, &self.exclude)?;
        
        let mut bsa = open(&self.file, &self.overrides)?;

        match bsa.dirs()? {
            SomeBsaRoot::Dirs(dirs) => {
                for dir in dirs {
                    for file in &dir {
                        let file_path = format!("{}/{}", dir.id(), file.id());
                        if matcher.matches(&file_path) {
                            println!("{}", file_path);
                            let mut out = open_output_file(&self.output, &[dir.id(), file.id()])?;
                            bsa.extract(&file, &mut out)?;
                        }
                    }
                }
            },
            SomeBsaRoot::Files(files) => {
                for file in files {
                    let file_path = format!("{}", file.id());
                    if matcher.matches(&file_path) {
                        println!("{}", file_path);
                        let mut out = open_output_file(&self.output, &[file.id()])?;
                        bsa.extract(&file, &mut out)?;
                    }
                }
            },
        }

        Ok(())
    }
}

fn open(file: &PathBuf, overrides: &Overrides) -> Result<SomeBsaReader<BufReader<File>>> {
    if let Some(vs) = &overrides.force_version {
        Version::from(vs).open(file)
    } else {
        bsalib::open(file)
    }
}

fn open_output_file(out: &PathBuf, ids: &[EntryId]) -> Result<File> {
    let mut path = out.clone();
    for id in ids {
        path.push(as_path(id));
    }
    if let Some(parent) = path.parent(){
        fs::create_dir_all(parent)?;
    }
    check_exists(&path)?;
    File::create(path)
}

fn as_path(id: &EntryId) -> PathBuf {
    if let Some(name) = &id.name {
        let mut path = PathBuf::new();
        for part in name.split("\\") {
            path.push(part);
        }
        path
    } else {
        PathBuf::from(format!("{}", id.hash))
    }
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
        if Version::from(&self.version) != Version::V10X(Version10X::V105) {
            return Err(Error::new(ErrorKind::Unsupported, "currently only v105 is supported"))
        }

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
        
        let dirs = list_dir(&self.file)?;
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
