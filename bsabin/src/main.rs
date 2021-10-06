use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{BufReader, Result, Error, ErrorKind};
use clap::Clap;
use glob::{Pattern, MatchOptions};
use thiserror::Error;

use bsalib::{SomeBsaReader, SomeBsaRoot, Version, V001, V105};
use bsalib::write::{BsaWriter, list_dir};
use bsalib::read::{BsaReader, BsaEntry, EntryId};
use bsalib::v105;
use bsalib;

mod cli;
use crate::cli::{Cmds, Info, List, Extract, Create, Overrides, CreateArgs};


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
            println!("{:?}", bsa.header());
        } else {
            println!("{}", bsa.header());
        }
        Ok(())
    }
}

impl Cmd for List {
    fn exec(&self) -> Result<()> {
        let mut bsa = open(&self.file, &self.overrides)?;
        match bsa.list()? {
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

        match bsa.list()? {
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
        let output = match self.output.as_ref() {
            Some(p) => p.clone(),
            None => {
                let mut tmp = (&self).file.clone();
                tmp.set_extension("bsa");
                tmp.to_owned()
            },
        };

        check_exists(&output)?;
        let dirs = list_dir(&self.file)?;
        let file = File::create(output)?;

        match &self.args {
            CreateArgs::V001 => {
                V001::write_bsa((), dirs, file)
                    .map_err(|err| Error::new(ErrorKind::Other, err))?;
            },
            CreateArgs::V105(args) => {
                let mut opts = v105::BsaWriterOptions::default();
                if args.compress {
                    opts.archive_flags |= v105::ArchiveFlag::CompressedArchive;
                }
                
                if args.embed_file_names {
                    opts.archive_flags |= v105::ArchiveFlag::EmbedFileNames;
                }
                V105::write_bsa(opts, dirs, file)?;
            },
            v => print!("unsupported version: {}", Version::from(v)),
        }
        Ok(())
    }
}
