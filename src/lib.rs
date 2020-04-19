use std::path::{Path, PathBuf};

/// Normalize a target path to an absolute path relative to a base
/// directory (typically the current working directory) without
/// accessing the file system
///
/// # Arguments
///
/// * `base_dir` - Base directory (must be absolute), typically the current working directory
/// * `target_path` - Target path
///
/// # Examples
///
/// ```
/// ```
pub fn absolute_path<P0: AsRef<Path>, P1: AsRef<Path>>(
    base_dir: P0,
    path: P1,
) -> std::io::Result<PathBuf> {
    if !base_dir.as_ref().is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Base directory {} is not absolute",
                base_dir.as_ref().display()
            ),
        ));
    }
    Ok(base_dir.as_ref().join(path))
}

#[cfg(test)]
mod tests {
    use super::absolute_path;

    use std::path::Path;

    #[test]
    fn nonabsolute_base_dir_fails() {
        check_absolute_path_fails("aa/bb/cc", "")
    }

    #[test]
    fn normalized_base_dir_relative_path_empty() {
        check_absolute_path("/aa/bb/cc", "", "/aa/bb/cc/")
    }

    #[test]
    fn normalized_base_dir_relative_path_single_part() {
        check_absolute_path("/aa/bb/cc", "dd", "/aa/bb/cc/dd")
    }

    #[test]
    fn normalized_base_dir_relative_path_multiple_parts() {
        check_absolute_path("/aa/bb/cc", "dd/ee", "/aa/bb/cc/dd/ee")
    }

    fn check_absolute_path(p0: &str, p1: &str, expected: &str) {
        let p = absolute_path(Path::new(p0), Path::new(p1)).unwrap();
        assert!(p.is_absolute());
        assert_eq!(p.to_str().unwrap(), expected)
    }

    fn check_absolute_path_fails(p0: &str, p1: &str) {
        assert!(absolute_path(Path::new(p0), Path::new(p1)).is_err())
    }
}
