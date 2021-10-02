use std::path::PathBuf;
use clap::Clap;
use glob::Pattern;
use bsa::version::Version;


#[derive(Debug, Clap)]
#[clap(about = "Bethesda Softworks Archive tool")]
pub enum Cmds {
    Info(Info),
    List(List),
    Extract(Extract),
    Create(Create),
}

#[derive(Debug, Clap)]
#[clap()]
pub struct Info {
    #[clap(short, long)]
    pub verbose: bool,

    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

#[derive(Debug, Clap)]
#[clap()]
pub struct List {
    #[clap(short, long)]
    pub attributes: bool,

    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

#[derive(Debug, Clap)]
#[clap()]
pub struct Extract {
    #[clap(short, long, parse(from_os_str), default_value=".")]
    pub output: PathBuf,

    #[clap(parse(from_os_str))]
    pub file: PathBuf,

    #[clap(parse(try_from_str))]
    pub paths: Vec<Pattern>,
}

#[derive(Debug, Clap)]
#[clap()]
pub struct Create {
    #[clap()]
    pub version: Version,

    #[clap(short, long)]
    pub compress: bool,

    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
    
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}
