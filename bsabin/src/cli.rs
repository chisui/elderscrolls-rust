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
    #[clap(aliases = &["a"])]
    Add(Add),
    #[clap(aliases = &["m"])]
    Merge(Merge),
    #[clap(aliases = &["d", "r", "remove"])]
    Del(Del),
}

/// Print information about an archive file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Info {
    #[clap(flatten)]
    pub open_opts: OpenOpts,

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
    pub open_opts: OpenOpts,

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
    pub open_opts: OpenOpts,

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

#[derive(Debug, Clap)]
#[clap()]
pub struct Add {
    /// add the file(s) compressed.
    #[clap(short, long)]
    pub compress: bool,

    /// The archive file to add to.
    #[clap(parse(from_os_str))]
    pub output: PathBuf,
    
    /// Files to add.
    #[clap(parse(from_os_str))]
    pub file: Vec<PathBuf>,
}

/// merge multiple archives. settings and flags are taken from the first file.
#[derive(Debug, Clap)]
#[clap()]
pub struct Merge {
    /// Archives to merge
    #[clap(parse(from_os_str))]
    pub file: Vec<PathBuf>,
}


/// Remove files from an archive.
#[derive(Debug, Clap)]
#[clap()]
pub struct Del {
    /// Archive to delet from.
    #[clap(parse(from_os_str))]
    pub file: PathBuf,

    /// glob patterns of files to remove.
    #[clap(parse(try_from_str))]
    pub files: Vec<Pattern>,
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
    /// don't include direcotry names.
    /// Games may not load achrives with this option.
    #[clap(long)]
    pub no_dir_names: bool,

    /// don't include file names.
    /// Games may not load achrives with this option.
    #[clap(long)]
    pub no_file_names: bool,

    /// Compress files.
    #[clap(short, long)]
    pub compress: bool,

    /// set the retain directories names flag.
    /// This has no effect on the file structure.
    /// May have unknown effect in games.
    #[clap(long)]
    pub retain_dir_names: bool,

    /// set the retain file names flag.
    /// This has no effect on the file structure.
    /// May have unknown effect in games.
    #[clap(long)]
    pub retain_file_names: bool,

    /// create an Xbox360 compatible archive.
    #[clap(long)]
    pub xbox: bool,

    /// Embed the filenames with the data.
    #[clap(long)]
    pub embed_file_names: bool,
}

#[derive(Debug, Clap)]
pub struct OpenOpts {
    /// Ignore file version information and treat it as this version instead.
    #[clap(arg_enum, long)]
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
