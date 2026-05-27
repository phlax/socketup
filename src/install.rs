use std::fs;
use std::io;
use std::path::Path;

pub fn install_self(dst: &Path) -> io::Result<u64> {
    let src = std::env::current_exe()?;
    fs::copy(src, dst)
}
