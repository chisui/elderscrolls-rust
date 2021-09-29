use std::io::{BufReader, Result};
use std::fs::{self, File};
use std::str::FromStr;
use std::path::{Path, PathBuf};
use clap::Clap;
use glob::{Pattern, MatchOptions};

use bsa;
use bsa::bzstring::NullTerminated;
use bsa::bin::{self, Writable};
use bsa::version::{Version, Version10X};
use bsa::v105;
use bsa::BsaArchive;
use bsa::archive::{Bsa, FileId};

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
        let bsa = BsaArchive::open(&mut reader)?;
        println!("{}", bsa);
        Ok(())
    }
}

impl Cmd for List {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let mut bsa = BsaArchive::open(&mut reader)?;
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
        let matcher = FileMatcher {
            patterns: self.paths.clone()
        };

        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let mut bsa = BsaArchive::open(&mut reader)?;

        let dirs = bsa.read_dirs()?;
        for dir in dirs {
            for file in dir.files {
                let file_path = format!("{}/{}", dir.name, file.name);
                if matcher.matches(&file_path) {
                    println!("{}", file_path);
                    let mut out = open_output_file(&self.output, &dir.name, &file.name)?;
                    bsa.extract(file, &mut out)?;
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
    File::create(path_buf.as_path())
}


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

        if output.exists() {
            println!("{} already exists", output.to_string_lossy());
            return Ok(())
        }

        let mut file = File::create(output)?;
        let mut writer = v105::BsaWriter::new(file,
            v105::BsaWriterOptions::default())?;
        
        let dirs = list_dir(&self.file)?;
        for (dir, files) in &dirs {
            println!("{} ->", dir.to_string_lossy());
            for file in files {
                println!("    {}", file.file_name().to_string_lossy());
            }
        }
        let file_names: Vec<NullTerminated> = dirs.iter()
            .flat_map(|(_, files)| files)
            .map(|f| NullTerminated::from_str(&f.file_name().to_string_lossy()).unwrap())
            .collect();
        let mut header = v105::Header::default();
        header.folder_count = dirs.len() as u32;
        header.file_count = file_names.len() as u32;
        header.total_file_name_length = bin::size_many(&file_names) as u32;
        header.total_folder_name_length = dirs.iter()
            .map(|(dir, _)| (dir.to_string_lossy().len() as u32) + 1)
            .sum();
        header.write_here(writer)?;
        println!("{:#?}", header);
        Ok(())
    }
}

fn list_dir(dir: &Path) -> Result<Vec<(Box<Path>, Vec<fs::DirEntry>)>> {
    let mut stack = vec![PathBuf::new()];
    let mut res = vec![];
    while let Some(path) = stack.pop() {
        let mut files = vec![];
        let pwd: PathBuf = [dir, &path].iter().collect();
        for e in fs::read_dir(pwd)? {
            let entry = e?;
            if entry.file_type()?.is_dir() {
                stack.push([&path, &PathBuf::from(entry.file_name())].iter().collect());
            } else {
                files.push(entry);
            }
        }
        if !files.is_empty() {
            res.push((path.into(), files));
        }
    }
    Ok(res)
}
