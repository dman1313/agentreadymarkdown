// AZW3 (KF8) and AZW — Kindle PDB via shared `kindle` module (MIT `mobi` crate).
use std::path::Path;

use crate::converters::kindle::{self, KindleFormat};
use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

pub fn convert_azw3(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    kindle::convert_kindle(path, KindleFormat::Azw3)
}

pub fn convert_azw(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    kindle::convert_kindle(path, KindleFormat::Azw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nonexistent_azw3() {
        assert!(convert_azw3(Path::new("/tmp/nonexistent-agentready.azw3")).is_err());
    }

    #[test]
    fn rejects_invalid_azw3_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.azw3");
        std::fs::write(&path, b"not kindle").unwrap();
        assert!(convert_azw3(&path).is_err());
    }

    #[test]
    fn rejects_invalid_azw_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.azw");
        std::fs::write(&path, b"not kindle").unwrap();
        assert!(convert_azw(&path).is_err());
    }
}
