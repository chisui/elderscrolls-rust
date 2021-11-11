use std::io::{Read, Result};

use bytemuck::Pod;


pub trait ReadStructExt<S>
where S: Sized {
    fn read_struct(&mut self) -> Result<S>;
}

impl<S, R> ReadStructExt<S> for R
where S: Pod, R: Read {
    fn read_struct(&mut self) -> Result<S> {
        let mut val = S::zeroed();
        let slice = bytemuck::bytes_of_mut(&mut val);
        self.read_exact(slice)?;
        Ok(val)
    }
}
