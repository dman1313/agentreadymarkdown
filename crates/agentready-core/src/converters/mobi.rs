// MOBI conversion — delegates to shared Kindle PDB pipeline.
use std::path::Path;

use crate::converters::kindle::{self, KindleFormat};
use crate::converters::ConversionResult;
use crate::models::AgentReadyError;

pub fn convert_mobi(path: &Path) -> Result<ConversionResult, AgentReadyError> {
    kindle::convert_kindle(path, KindleFormat::Mobi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rejects_nonexistent_file() {
        assert!(convert_mobi(Path::new("/tmp/nonexistent-agentready.mobi")).is_err());
    }

    #[test]
    fn rejects_invalid_mobi_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.mobi");
        std::fs::write(&path, b"not a mobi file").unwrap();
        assert!(convert_mobi(&path).is_err());
    }
}
