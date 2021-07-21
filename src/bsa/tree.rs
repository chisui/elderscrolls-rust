

pub enum BsaFile<N> {
    File {
        pub name: N,
        pub compressed: bool,
        pub offset: u64,
    }
    DirDir {
        pub name: N,
        pub files: Vec<BsaFile<N>>,
    }
}