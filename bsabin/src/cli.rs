use std::path::PathBuf;
use clap::{ArgEnum, Clap};
use glob::Pattern;
use bsalib::{Version, Version10X};


#[derive(Debug, Clap)]
#[clap(about = "Bethesda Softworks Archive tool")]
pub enum Cmds {
    Info(Info),
    List(List),
    Extract(Extract),
    Create(Create),
}

/// Print information about an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Info {
    /// Print all header informations.
    #[clap(short, long)]
    pub verbose: bool,

    /// The archive file.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

/// List files in an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct List {
    /// print file attributes. This includes the size and whether or not the file is compressed.
    #[clap(short, long)]
    pub attributes: bool,

    /// The archive file.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}

/// Extract files from an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Extract {
    /// Glob patterns that all file names that should be extracted have to match.
    #[clap(short, long, parse(try_from_str))]
    pub include: Vec<Pattern>,

    /// Glob patterns that no file name that should be extracted should to match.
    #[clap(short, long, parse(try_from_str))]
    pub exclude: Vec<Pattern>,

    /// The archive file.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,

    /// Output directory. If none is provided the current directory is used.
    #[clap(parse(from_os_str), default_value=".")]
    pub output: PathBuf,
}

#[derive(ArgEnum, Debug, PartialEq, Clone)]
pub enum VersionSlug {
    V001, Tes3, Morrowind,
    V103, Tes4, Oblivion,
    V104, Tes5, Skyrim, Fallout3, F3, Fnv, NewVegas, FalloutNewVegas,
    V105, Tes5se, SkyrimSE,
    V200, Fallout4, F4, Fallout76, F76,
}
use VersionSlug::*;
impl From<&VersionSlug> for Version {
    fn from(slug: &VersionSlug) -> Self {
        match slug {
            V001 | Tes3 | Morrowind => Version::V001,
            V103 | Tes4 | Oblivion => Version::V10X(Version10X::V103),
            V104 | Tes5 | Skyrim | Fallout3 | F3 | Fnv | NewVegas | FalloutNewVegas => Version::V10X(Version10X::V104),
            V105 | Tes5se | SkyrimSE => Version::V10X(Version10X::V105),
            V200 | Fallout4 | F4 | Fallout76 | F76 => Version::V200(0),
        }
    }
}

/// Create an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Create {
    /// bsa archive Version or game name.
    #[clap(arg_enum)]
    pub version: VersionSlug,

    /// Compress files.
    #[clap(short, long)]
    pub compress: bool,
    
    /// Embed the filenames with the data.
    #[clap(long="embed-file-names")]
    pub embed_file_names: bool,

    /// The archive file to create. If none is provided the directory name plus ".bsa" is used.
    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
    
    /// Root directory of the archive to create.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}
