use sha1::{Digest, Sha1};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceDescriptor {
    pub kind: String,
    pub locator: String,
}

impl SourceDescriptor {
    pub fn new(kind: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            locator: locator.into(),
        }
    }

    pub fn is_source_backed(&self) -> bool {
        !matches!(
            self.kind.as_str(),
            "harness-local" | "shared-store" | "unmanaged-local"
        )
    }
}

pub fn stable_id(parts: &[&str]) -> String {
    let mut digest = Sha1::new();
    for part in parts {
        digest.update(part.as_bytes());
        digest.update([0u8]);
    }
    format!("{:x}", digest.finalize())[..12].to_string()
}
