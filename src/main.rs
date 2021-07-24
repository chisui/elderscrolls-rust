use std::io::{BufReader, Result};
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use glob::{Pattern, MatchOptions};

use bsa::Bsa;


#[derive(Debug, StructOpt)]
#[structopt(about = "Bethesda Softworks Archive tool")]
enum SubCmd {
    Info(InfoCmd),
    List(ListCmd),
    Extract(ExtractCmd),
}
trait Cmd {
    fn exec(self) -> Result<()>;
}

fn main() -> Result<()> {
    let cmd = SubCmd::from_args();
    match cmd {
        SubCmd::Info(cmd)    => cmd.exec(),
        SubCmd::List(cmd)    => cmd.exec(),
        SubCmd::Extract(cmd) => cmd.exec(),
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct InfoCmd {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}
impl Cmd for InfoCmd {
    fn exec(self) -> Result<()> {
        let mut reader = open(self.file)?;
        let bsa = Bsa::open(&mut reader)?;
        println!("{}", bsa);
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct ListCmd {        
    #[structopt(short, long)]
    attributes: bool,

    #[structopt(parse(from_os_str))]
    file: PathBuf,
}
impl Cmd for ListCmd {
    fn exec(self) -> Result<()> {
        let mut reader = open(self.file)?;

        let bsa = Bsa::open(&mut reader)?;
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
struct ExtractCmd {
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
impl Cmd for ExtractCmd {
    fn exec(self) -> Result<()> {
        let mut reader = open(self.file)?;

        let bsa = Bsa::open(&mut reader)?;
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
                    bsa.extract(file, path_buf.as_path(), &mut reader)?;
                }
            }
        }

        Ok(())
    }
}


fn open(file: PathBuf) -> Result<BufReader<File>> {
    let file = File::open(file)?;
    Ok(BufReader::new(file))
}
