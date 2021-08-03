#![feature(macro_attributes_in_derive_output)]
use std::io::{BufReader, Result};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use glob::{Pattern, MatchOptions};

use bsa;
use bsa::version::Version;
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
        println!("{:?}", self);
        Ok(())
    }
}
