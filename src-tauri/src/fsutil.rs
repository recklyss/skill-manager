use std::path::{Path, PathBuf};

/// Write `bytes` to `path` atomically: create parent dirs, write to a sibling
/// `<path>.tmp` file, then rename over the target.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    let temp = PathBuf::from(temp);
    std::fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&temp, path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Recursively copy the contents of `src` into `dst`, creating `dst` if needed.
pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_creates_parents_and_leaves_no_temp() {
        let base = std::env::temp_dir().join(format!("fsutil-aw-{}", std::process::id()));
        let target = base.join("nested").join("out.json");
        atomic_write(&target, b"hello").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "hello");
        assert!(!base.join("nested").join("out.json.tmp").exists());
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn copy_dir_all_copies_nested_tree() {
        let base = std::env::temp_dir().join(format!("fsutil-cp-{}", std::process::id()));
        let src = base.join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), b"a").unwrap();
        std::fs::write(src.join("sub").join("b.txt"), b"b").unwrap();
        let dst = base.join("dst");
        copy_dir_all(&src, &dst).unwrap();
        assert_eq!(std::fs::read_to_string(dst.join("a.txt")).unwrap(), "a");
        assert_eq!(std::fs::read_to_string(dst.join("sub").join("b.txt")).unwrap(), "b");
        std::fs::remove_dir_all(&base).ok();
    }
}
