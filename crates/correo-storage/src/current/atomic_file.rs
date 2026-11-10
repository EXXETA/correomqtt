use std::io::{self, Write};
use std::path::Path;

use atomic_write_file::AtomicWriteFile;

pub fn write_file_atomic(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = AtomicWriteFile::open(path)?;
    file.write_all(content)?;
    file.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_content() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("payload.bin");
        std::fs::write(&path, b"old").unwrap();

        write_file_atomic(&path, b"new").unwrap();

        assert_eq!(std::fs::read(path).unwrap(), b"new");
    }
}
