use path_clean::clean;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

/// Normalize a target path to an absolute path relative to a base
/// directory (typically the current working directory) without
/// accessing the file system
///
/// # Arguments
///
/// * `base_dir` - Base directory (must be absolute), typically the current working directory
/// * `path` - Path
pub fn absolute_path<B: AsRef<Path>, P: AsRef<Path>>(base_dir: B, path: P) -> Result<PathBuf> {
    fn normalize(path: &Path) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        fn platform_clean(path: &str) -> String {
            use std::path::MAIN_SEPARATOR;

            clean(&path.replace(MAIN_SEPARATOR, "/")).replace('/', &MAIN_SEPARATOR.to_string())
        }
        #[cfg(not(target_os = "windows"))]
        fn platform_clean(path: &str) -> String {
            clean(path)
        }

        path.to_str()
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Other,
                    format!("Path {} cannot be converted to string", path.display()),
                )
            })
            .map(platform_clean)
            .map(PathBuf::from)
    }

    if !base_dir.as_ref().is_absolute() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "Base directory {} is not absolute",
                base_dir.as_ref().display()
            ),
        ));
    }

    normalize(&match path.as_ref().components().count() {
        0 => base_dir.as_ref().to_path_buf(),
        _ => base_dir.as_ref().join(path),
    })
}

#[cfg(test)]
mod tests {
    use asserts::*;
    use helpers::*;

    #[test]
    fn fails_if_base_dir_not_absolute() {
        check_absolute_path_fails(abs("aa/bb/cc"), rel(""))
    }

    #[test]
    fn path_empty() {
        check_absolute_path(abs("/aa/bb/cc"), rel(""), "/aa/bb/cc", 3)
    }

    #[test]
    fn base_dir_unnormalized_path_empty() {
        check_absolute_path(abs("/aa/../bb/cc"), rel(""), "/bb/cc", 2)
    }

    #[test]
    fn path_single_component_relative() {
        check_absolute_path(abs("/aa/bb/cc"), rel("dd"), "/aa/bb/cc/dd", 4)
    }

    #[test]
    fn path_single_component_absolute() {
        check_absolute_path(abs("/aa/bb/cc"), abs("/dd"), "/dd", 1)
    }

    #[test]
    fn path_multiple_components_relative() {
        check_absolute_path(abs("/aa/bb/cc"), rel("dd/ee"), "/aa/bb/cc/dd/ee", 5)
    }

    #[test]
    fn path_multiple_components_absolute() {
        check_absolute_path(abs("/aa/bb/cc"), abs("/dd/ee"), "/dd/ee", 2)
    }

    #[test]
    fn path_multiple_components_unnormalized() {
        check_absolute_path(abs("/aa/bb/cc"), rel("dd/../ee"), "/aa/bb/cc/ee", 4)
    }

    #[test]
    fn both_unnormalized() {
        check_absolute_path(abs("/aa/bb/../cc"), rel("dd/../ee"), "/aa/cc/ee", 3)
    }

    mod asserts {
        use crate::absolute_path;

        use super::helpers::*;
        use super::platform_helpers::*;

        pub fn check_absolute_path(
            base_dir: TestPath,
            path: TestPath,
            expected_path_str: &str,
            expected_component_count: usize,
        ) {
            let p = absolute_path(from_test_path(base_dir), from_test_path(path)).unwrap();
            assert!(p.is_absolute());
            assert_eq!(p, from_test_path(abs(expected_path_str)));
            assert_eq!(
                p.to_str().unwrap(),
                from_test_path(abs(expected_path_str)).to_str().unwrap()
            );
            assert_eq!(path_component_count(&p).unwrap(), expected_component_count);
            assert!(!p.to_str().unwrap().contains(OTHER_SEPARATOR))
        }

        pub fn check_absolute_path_fails(p0: TestPath, p1: TestPath) {
            assert!(absolute_path(from_test_path(p0), from_test_path(p1)).is_err())
        }
    }

    pub mod helpers {
        use self::TestPath::*;

        pub enum TestPath {
            Abs(String),
            Rel(String),
        }

        pub fn abs(s: &str) -> TestPath {
            Abs(String::from(s))
        }

        pub fn rel(s: &str) -> TestPath {
            Rel(String::from(s))
        }
    }

    #[cfg(target_os = "windows")]
    pub mod platform_helpers {
        use std::path::Component::*;
        use std::path::Prefix::*;
        use std::path::{Path, PathBuf};

        use super::helpers::TestPath::{self, *};

        pub const OTHER_SEPARATOR: char = '/';

        pub fn from_test_path(test_path: TestPath) -> PathBuf {
            let raw = match test_path {
                Abs(s) => format!(
                    "Z:{}",
                    s.replace('/', &std::path::MAIN_SEPARATOR.to_string())
                ),
                Rel(s) => s.replace('/', &std::path::MAIN_SEPARATOR.to_string()),
            };
            PathBuf::from(raw)
        }

        pub fn path_component_count<P: AsRef<Path>>(path: P) -> Option<usize> {
            let mut iter = path.as_ref().components();

            match iter.next() {
                Some(Prefix(prefix_component)) => match prefix_component.kind() {
                    Disk(90) => {}
                    _ => return None,
                },
                _ => return None,
            };

            match iter.next() {
                Some(RootDir) => {}
                _ => return None,
            };

            let mut n = 0;
            loop {
                match iter.next() {
                    Some(Normal(_)) => n += 1,
                    Some(_) => return None,
                    None => return Some(n),
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub mod platform_helpers {
        use std::path::Component::*;
        use std::path::{Path, PathBuf};

        use super::helpers::TestPath::{self, *};

        pub const OTHER_SEPARATOR: char = '\\';

        pub fn from_test_path(test_path: TestPath) -> PathBuf {
            let raw = match test_path {
                Abs(s) => s,
                Rel(s) => s,
            };
            PathBuf::from(raw)
        }

        pub fn path_component_count<P: AsRef<Path>>(path: P) -> Option<usize> {
            let mut iter = path.as_ref().components();

            match iter.next() {
                Some(RootDir) => {}
                _ => return None,
            };

            let mut n = 0;
            loop {
                match iter.next() {
                    Some(Normal(_)) => n += 1,
                    Some(_) => return None,
                    None => return Some(n),
                }
            }
        }
    }
}
