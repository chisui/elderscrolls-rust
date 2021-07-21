use std::io::Result;

use super::version::Version;


pub trait Bsa {
    fn version(&self) -> Result<Version>;
}
