use std::io::{BufReader, Result};
use std::fs::{self, File};
use std::str::FromStr;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use glob::{Pattern, MatchOptions};

use bsa;
use bsa::bzstring::NullTerminated;
use bsa::bin::{self, Writable};
use bsa::version::{Version, Version10X};
use bsa::v105;
use bsa::SomeBsa;
use bsa::archive::Bsa;


#[derive(Debug, StructOpt)]
#[structopt(about = "Bethesda Softworks Archive tool")]
enum Cmds {
    Info(Info),
    List(List),
    Extract(Extract),
    Create(Create),
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
fn main() -> Result<()> {
    Cmds::from_args().exec()
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Info {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}
impl Cmd for Info {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;
        let bsa = SomeBsa::open(&mut reader)?;
        println!("{}", bsa);
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct List {        
    #[structopt(short, long)]
    attributes: bool,

    #[structopt(parse(from_os_str))]
    file: PathBuf,
}
impl Cmd for List {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let bsa = SomeBsa::open(&mut reader)?;
        let dirs = bsa.read_dirs(&mut reader)?;
        for dir in dirs {
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

#[derive(Debug, StructOpt)]
#[structopt()]
struct Extract {
    #[structopt(short, long, parse(from_os_str), default_value=".")]
    output: PathBuf,
    
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    
    #[structopt(parse(try_from_str))]
    paths: Vec<Pattern>,
}
fn should_extract(paths: &Vec<Pattern>, path: &String) -> bool {
    let match_opt = MatchOptions {
        case_sensitive: false,
        require_literal_leading_dot: false,
        require_literal_separator: false,
    };
    paths.is_empty()
        || paths.iter().any(|p|
            p.matches_with(&path, match_opt)
            || path.starts_with(p.as_str()))
}
impl Cmd for Extract {
    fn exec(&self) -> Result<()> {
        let mut reader = File::open(&self.file)
            .map(BufReader::new)?;

        let bsa = SomeBsa::open(&mut reader)?;
        let dirs = bsa.read_dirs(&mut reader)?;

        fs::create_dir_all(&self.output)?;

        for dir in dirs {
            for file in dir.files {
                let file_path = format!("{}/{}", dir.name, file.name);
                if should_extract(&self.paths, &file_path) {
                    println!("{}", file_path);
                    let mut path_buf = PathBuf::from(&self.output);
                    path_buf.push(format!("{}", dir.name));
                    fs::create_dir_all(&path_buf)?;
                    path_buf.push(format!("{}", file.name));
                    let mut writer = File::create(path_buf.as_path())?;
                    bsa.extract(file, &mut reader, &mut writer)?;
                }
            }
        }

        Ok(())
    }
}


#[derive(Debug, StructOpt)]
#[structopt()]
struct Create {
    #[structopt()]
    version: Version,

    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    
    #[structopt(parse(from_os_str))]
    file: PathBuf,
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

        let mut writer = File::create(output)?;
        Version::V10X(Version10X::V105).write_here(&mut writer)?;

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
