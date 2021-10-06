use std::path::PathBuf;
use clap::{ArgEnum, Clap};
use glob::Pattern;
use bsalib::{Version, Version10X, BA2Type};


#[derive(Debug, Clap)]
#[clap(about = "Bethesda Softworks Archive tool")]
pub enum Cmds {
    #[clap(aliases = &["i"])]
    Info(Info),
    #[clap(aliases = &["l", "ls", "lst", "dir"])]
    List(List),
    #[clap(aliases = &["x"])]
    Extract(Extract),
    #[clap(aliases = &["c"])]
    Create(Create),
}

/// Print information about an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Info {
    #[clap(flatten)]
    pub overrides: Overrides,

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
    #[clap(flatten)]
    pub overrides: Overrides,

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
    #[clap(flatten)]
    pub overrides: Overrides,

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

/// Create an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Create {
    /// bsa archive Version or game name.
    #[clap(subcommand)]
    pub args: CreateArgs,

    /// The archive file to create. If none is provided the directory name plus ".bsa" is used.
    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,
    
    /// Root directory of the archive to create.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,
}



#[derive(Debug, PartialEq, Clone, Clap)]
pub enum CreateArgs {
    #[clap(aliases = &["001", "tes3", "morrowind"])]
    V001,
    #[clap(aliases = &["103", "tes4", "oblivion"])]
    V103(V10XCreateArgs),
    #[clap(aliases = &["104", "tes5", "skyrim", "fallout3", "f3", "fnv", "newvegas", "falloutnewvegas"])]
    V104(V10XCreateArgs),
    #[clap(aliases = &["105", "tes5se", "skyrimse"])]
    V105(V10XCreateArgs),
    #[clap(aliases = &["2", "200", "ba2", "fallout4", "f4", "fallout76", "f76"])]
    BA2,
}
impl From<&CreateArgs> for Version {
    fn from(slug: &CreateArgs) -> Self {
        match slug {
            CreateArgs::V001 => Version::V001,
            CreateArgs::V103(_) => Version::V10X(Version10X::V103),
            CreateArgs::V104(_) => Version::V10X(Version10X::V104),
            CreateArgs::V105(_) => Version::V10X(Version10X::V105),
            CreateArgs::BA2  => Version::BA2(BA2Type::BTDX, 0),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Clap)]
pub struct V10XCreateArgs {
    /// Compress files.
    #[clap(short, long)]
    pub compress: bool,
    
    /// Embed the filenames with the data.
    #[clap(long="embed-file-names")]
    pub embed_file_names: bool,
}

#[derive(Debug, Clap)]
pub struct Overrides {
    /// Ignore file version information and treat it as this version instead.
    #[clap(arg_enum, long="force-version")]
    pub force_version: Option<VersionSlug>,
}


#[derive(ArgEnum, Debug, PartialEq, Clone)]
pub enum VersionSlug {
    #[clap(aliases = &["001", "tes3", "morrowind"])]
    V001,
    #[clap(aliases = &["103", "tes4", "oblivion"])]
    V103,
    #[clap(aliases = &["104", "tes5", "skyrim", "fallout3", "f3", "fnv", "newvegas", "falloutnewvegas"])]
    V104,
    #[clap(aliases = &["105", "tes5se", "skyrimse"])]
    V105,
    #[clap(aliases = &["2", "200", "ba2", "fallout4", "f4", "fallout76", "f76"])]
    BA2,
}
impl From<&VersionSlug> for Version {
    fn from(slug: &VersionSlug) -> Self {
        match slug {
            VersionSlug::V001 => Version::V001,
            VersionSlug::V103 => Version::V10X(Version10X::V103),
            VersionSlug::V104 => Version::V10X(Version10X::V104),
            VersionSlug::V105 => Version::V10X(Version10X::V105),
            VersionSlug::BA2  => Version::BA2(BA2Type::BTDX, 0),
        }
    }
}
