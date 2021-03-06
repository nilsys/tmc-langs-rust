//! Contains the Python3 student file policy

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tmc_langs_framework::policy::StudentFilePolicy;

pub struct Python3StudentFilePolicy {
    config_file_parent_path: PathBuf,
}

impl Python3StudentFilePolicy {
    pub fn new(config_file_parent_path: PathBuf) -> Self {
        Self {
            config_file_parent_path,
        }
    }
}

impl StudentFilePolicy for Python3StudentFilePolicy {
    fn get_config_file_parent_path(&self) -> &Path {
        &self.config_file_parent_path
    }

    fn is_student_source_file(&self, path: &Path) -> bool {
        path.starts_with("src")
            && path.extension() != Some(OsStr::new("pyc"))
            && !path
                .components()
                .any(|c| c.as_os_str() == OsStr::new("__pycache__"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn in_src_is_source_file() {
        let policy = Python3StudentFilePolicy::new(PathBuf::from(""));
        assert!(policy.is_student_source_file(Path::new("src/some_file.py")));
    }

    #[test]
    fn pycache_is_not_source_file() {
        let policy = Python3StudentFilePolicy::new(PathBuf::from(""));
        assert!(!policy.is_student_source_file(Path::new("__pycache__")));
        assert!(!policy.is_student_source_file(Path::new("__pycache__/cachefile")));
        assert!(!policy.is_student_source_file(Path::new("src/__pycache__")));
    }

    #[test]
    fn pyc_is_not_source_file() {
        let policy = Python3StudentFilePolicy::new(PathBuf::from(""));
        assert!(!policy.is_student_source_file(Path::new("some.pyc")));
        assert!(!policy.is_student_source_file(Path::new("src/other.pyc")));
    }
}
