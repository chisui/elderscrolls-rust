use std::io::{BufReader, Result};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

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
        let file = File::open(self.file).expect("file not found!");
        let mut buffer = BufReader::new(file);

        let bsa = Bsa::open(&mut buffer)?;
        match bsa {
            Bsa::V103(header) => {
                println!("BSA Version v103, used by: TES IV: Oblivion");
                println!("{}", header);
            },
            Bsa::V104(header) => {
                println!("BSA Version v104, used by: Fallout 3, Fallout: NV, TES V: Skyrim");
                println!("{}", header);
            },
            Bsa::V105(header) => {
                println!("BSA Version v105, used by: TES V: Skyrim Special Edition");
                println!("{}", header);
            },
        }
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
        let file = File::open(self.file).expect("file not found!");
        let mut buffer = BufReader::new(file);

        let bsa = Bsa::open(&mut buffer)?;
        let dirs = bsa.read_dirs(&mut buffer)?;
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
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    
    #[structopt(parse(from_os_str))]
    paths: Vec<PathBuf>,
}
impl Cmd for ExtractCmd {
    fn exec(self) -> Result<()> {
        let file = File::open(self.file).expect("file not found!");
        let mut buffer = BufReader::new(file);

        let bsa = Bsa::open(&mut buffer)?;
        let dirs = bsa.read_dirs(&mut buffer)?;



        Ok(())
    }
}