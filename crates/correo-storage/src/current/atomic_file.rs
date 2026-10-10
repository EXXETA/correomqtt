use std::io::{self, Write};
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

pub fn write_file_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = AtomicWriteFile::open(path)?;
    file.write_all(content)?;
    file.commit()
}

pub fn write_file_atomic_private(path: &Path, content: &[u8]) -> io::Result<()> {
    #[cfg(unix)]
    let mut file = {
        use atomic_write_file::unix::OpenOptionsExt as _;
        use std::os::unix::fs::OpenOptionsExt as _;

        let mut options = AtomicWriteFile::options();
        options.mode(0o600).preserve_mode(false);
        options.open(path)?
    };
    #[cfg(not(unix))]
    let mut file = AtomicWriteFile::open(path)?;

    file.write_all(content)?;
    file.commit()
}
