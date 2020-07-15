//! Contains StudentFilePolicy.

use super::{Result, TmcProjectYml};
use crate::TmcError;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Specifies which files are student files.
///
/// Student files are any files that are expected to be modified and/or created by the student.
/// That is, any files that should not be overwritten when when updating an already downloaded
/// exercise and any files that should be submitted to the server.
pub trait StudentFilePolicy {
    /// Determines whether a file is a student source file.
    ///
    /// A file should be considered a student source file if it resides in a location the student
    /// is expected to create his or her own source files in the general case. Any special cases
    /// are specified as ExtraStudentFiles in a separate configuration.
    ///
    /// For example in a Java project that uses Apache Ant, should return `true` for any files in the `src` directory.
    fn is_student_file(
        &self,
        path: &Path,
        project_root_path: &Path,
        tmc_project_yml: &TmcProjectYml,
    ) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        if path.file_name() == Some(OsStr::new(".tmcproject.yml")) {
            return Ok(false);
        }

        // try to strip project root prefix
        let relative = path.strip_prefix(project_root_path).unwrap_or(path);
        Ok(self.is_extra_student_file(path, tmc_project_yml)?
            || project_root_path == path
            || self.is_student_source_file(relative))
    }

    fn get_config_file_parent_path(&self) -> &Path;

    fn get_tmc_project_yml(&self) -> Result<TmcProjectYml> {
        Ok(TmcProjectYml::from(self.get_config_file_parent_path())?)
    }

    /// Determines whether a file is an extra student file.
    fn is_extra_student_file(&self, path: &Path, tmc_project_yml: &TmcProjectYml) -> Result<bool> {
        let absolute = path
            .canonicalize()
            .map_err(|e| TmcError::Canonicalize(path.to_path_buf(), e))?;
        for path in &tmc_project_yml.extra_exercise_files {
            let path = path
                .canonicalize()
                .map_err(|e| TmcError::Canonicalize(path.to_owned(), e))?;

            if path.is_dir() && (path == absolute || absolute.starts_with(path)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn is_student_source_file(&self, path: &Path) -> bool;

    /// Used to check for files which should always be overwritten.
    fn is_updating_forced(&self, path: &Path, tmc_project_yml: &TmcProjectYml) -> Result<bool> {
        let absolute = path
            .canonicalize()
            .map_err(|e| TmcError::Canonicalize(path.to_path_buf(), e))?;
        for force_update_path in &tmc_project_yml.force_update {
            let force_absolute = force_update_path
                .canonicalize()
                .map_err(|e| TmcError::Canonicalize(force_update_path.to_owned(), e))?;
            if (absolute == force_absolute || absolute.starts_with(&force_absolute))
                && force_absolute.is_dir()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub struct NothingIsStudentFilePolicy {}

impl StudentFilePolicy for NothingIsStudentFilePolicy {
    fn is_student_file(
        &self,
        _path: &Path,
        _project_root_path: &Path,
        _tmc_project_yml: &TmcProjectYml,
    ) -> Result<bool> {
        Ok(false)
    }

    fn get_config_file_parent_path(&self) -> &Path {
        Path::new("")
    }

    fn is_extra_student_file(
        &self,
        _path: &Path,
        _tmc_project_yml: &TmcProjectYml,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn is_student_source_file(&self, _path: &Path) -> bool {
        unimplemented!()
    }

    fn is_updating_forced(&self, _path: &Path, _tmc_project_yml: &TmcProjectYml) -> Result<bool> {
        Ok(false)
    }
}

pub struct EverythingIsStudentFilePolicy {
    config_file_parent_path: PathBuf,
}

impl EverythingIsStudentFilePolicy {
    pub fn new(config_file_parent_path: PathBuf) -> Self {
        Self {
            config_file_parent_path,
        }
    }
}

impl StudentFilePolicy for EverythingIsStudentFilePolicy {
    fn is_student_file(
        &self,
        _path: &Path,
        _project_root_path: &Path,
        _tmc_project_yml: &TmcProjectYml,
    ) -> Result<bool> {
        Ok(true)
    }

    fn get_config_file_parent_path(&self) -> &Path {
        &self.config_file_parent_path
    }

    fn is_extra_student_file(
        &self,
        _path: &Path,
        _tmc_project_yml: &TmcProjectYml,
    ) -> Result<bool> {
        unimplemented!()
    }

    fn is_student_source_file(&self, _path: &Path) -> bool {
        unimplemented!()
    }

    fn is_updating_forced(&self, _path: &Path, _tmc_project_yml: &TmcProjectYml) -> Result<bool> {
        Ok(false)
    }
}
