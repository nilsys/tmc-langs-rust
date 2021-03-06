use super::{get_language_plugin, Plugin, TmcError, TmcProjectYml};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::fs;
use std::io::Write;
use std::path::Path;
use tmc_langs_framework::io::file_util;
use walkdir::WalkDir;
use zip::{read::ZipFile, write::FileOptions, ZipWriter};

/// TmcParams is used to safely construct data for a .tmcparams file, which contains lines in the form of
/// export A=B
/// export C=(D, E, F)
/// the keys and values of the inner hashmap are validated to make sure they are valid as bash variables
#[derive(Debug, Default)]
pub struct TmcParams(HashMap<ShellString, TmcParam>);

impl TmcParams {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_string<S: AsRef<str>, T: AsRef<str>>(
        &mut self,
        key: S,
        value: T,
    ) -> Result<(), TmcError> {
        // validate key
        let key = {
            let key = key.as_ref();
            if !Self::is_valid_key(key) {
                return Err(TmcError::InvalidParam(key.to_string()));
            }
            ShellString(key.to_string())
        };

        // validate value
        let value = {
            let value = value.as_ref();
            if !Self::is_valid_value(value) {
                return Err(TmcError::InvalidParam(value.to_string()));
            }
            ShellString(value.to_string())
        };

        self.0.insert(key, TmcParam::String(value));
        Ok(())
    }

    pub fn insert_array<S: AsRef<str>, T: AsRef<str>>(
        &mut self,
        key: S,
        values: Vec<T>,
    ) -> Result<(), TmcError> {
        let key = {
            let key = key.as_ref();
            if Self::is_valid_key(key) {
                ShellString(key.to_string())
            } else {
                return Err(TmcError::InvalidParam(key.to_string()));
            }
        };

        let values = values
            .into_iter()
            .map(|s| {
                let s = s.as_ref();
                if Self::is_valid_value(s) {
                    Ok(ShellString(s.to_string()))
                } else {
                    Err(TmcError::InvalidParam(s.to_string()))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.0.insert(key, TmcParam::Array(values));
        Ok(())
    }

    fn is_valid_key<S: AsRef<str>>(string: S) -> bool {
        if string.as_ref().is_empty() {
            return false;
        }

        for c in string.as_ref().chars() {
            if !c.is_ascii_alphabetic() && c != '_' {
                return false;
            }
        }
        true
    }

    fn is_valid_value<S: AsRef<str>>(string: S) -> bool {
        if string.as_ref().is_empty() {
            return false;
        }

        for c in string.as_ref().chars() {
            if !c.is_ascii_alphabetic() && c != '_' && c != '-' {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ShellString(String);

/// .tmcparams variables can be strings or arrays
#[derive(Debug)]
enum TmcParam {
    String(ShellString),
    Array(Vec<ShellString>),
}

// the Display impl escapes the inner strings with shellwords
impl Display for TmcParam {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Array(v) => write!(
                f,
                "( {} )",
                v.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

// the Display impl escapes the inner string with shellwords
impl Display for ShellString {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", shellwords::escape(&self.0))
    }
}

pub fn prepare_submission(
    zip_path: &Path,
    target_path: &Path,
    toplevel_dir_name: Option<String>,
    tmc_params: TmcParams,
    clone_path: &Path,
    stub_zip_path: Option<&Path>,
    output_zip: bool,
) -> Result<(), TmcError> {
    log::debug!("preparing submission for {}", zip_path.display());

    fn useless_file_filter(entry: &ZipFile) -> bool {
        let files_to_filter = &[
            OsStr::new(".DS_Store"),
            OsStr::new("desktop.ini"),
            OsStr::new("Thumbs.db"),
            OsStr::new(".directory"),
            OsStr::new("__MACOSX"),
        ];
        files_to_filter.contains(&entry.sanitized_name().as_os_str())
    }

    let temp = tempfile::tempdir().map_err(TmcError::TempDir)?;
    let received_dir = temp.path().join("received");
    fs::create_dir(&received_dir).map_err(|e| TmcError::CreateDir(received_dir.clone(), e))?;

    // unzip submission to received dir
    log::debug!("unzipping submission");
    file_util::unzip(zip_path, &received_dir, useless_file_filter)?;

    // find project dir in unzipped files
    let project_root = file_util::find_project_root(&received_dir)?;
    let project_root = project_root.ok_or(TmcError::NoProjectDirInZip)?;

    let plugin = get_language_plugin(&project_root)?;
    let dest = temp.path().join(
        toplevel_dir_name
            .as_ref()
            .map(Path::new)
            .unwrap_or_else(|| Path::new("dest")),
    );

    // write tmc params
    log::debug!("writing .tmcparams");
    let tmc_params_path = dest.join(".tmcparams");
    let mut tmc_params_file = file_util::create_file(&tmc_params_path)?;
    for (key, value) in tmc_params.0 {
        // todo handle arrays, shell escapes
        let export = format!("export {}={}\n", key, value);
        log::debug!("{}", export);
        tmc_params_file
            .write_all(export.as_bytes())
            .map_err(|e| TmcError::Write(tmc_params_path.clone(), e))?;
    }

    // copy IDE files
    log::debug!("copying IDE files");
    for ide_dir in &[
        // netbeans
        "nbproject",
        // eclipse
        ".classpath",
        ".project",
        ".settings",
        // idea
        ".idea",
    ] {
        let dir_in_received = project_root.join(ide_dir);
        let dir_in_clone = clone_path.join(ide_dir);
        if dir_in_received.exists() {
            file_util::copy(dir_in_received, &dest)?;
        } else if dir_in_clone.exists() {
            file_util::copy(dir_in_clone, &dest)?;
        }
    }

    // if stub zip path was given, unzip and find its project root
    let stub_project_root = if let Some(stub_zip_path) = stub_zip_path {
        let stub_dir = temp.path().join("stub");
        file_util::unzip(stub_zip_path, &stub_dir, useless_file_filter)?;
        file_util::find_project_root(stub_dir)?
    } else {
        None
    };

    // copy files
    log::debug!("copying language specific files");
    let tests_dir = stub_project_root.as_deref().unwrap_or(clone_path);
    match plugin {
        Plugin::Maven(_) => {
            // maven

            // copy pom
            file_util::copy(clone_path.join("pom.xml"), &dest)?;

            // copy src main and test
            let main_path = clone_path.join("src/main");
            if main_path.exists() {
                file_util::copy(main_path, dest.join("src"))?;
            }
            let test_path = tests_dir.join("src/test");
            if test_path.exists() {
                file_util::copy(test_path, dest.join("src"))?;
            }

            // copy files from config
            let config = plugin.get_exercise_packaging_configuration(clone_path)?;
            for path in config.student_file_paths {
                let student_file = project_root.join(&path);
                if student_file.exists() {
                    let target = if let Some(parent) = path.parent() {
                        dest.join(parent)
                    } else {
                        dest.to_path_buf()
                    };
                    file_util::copy(student_file, target)?;
                }
            }
            for path in config.exercise_file_paths {
                let exercise_file = tests_dir.join(&path);
                if exercise_file.exists() {
                    let target = if let Some(parent) = path.parent() {
                        dest.join(parent)
                    } else {
                        dest.to_path_buf()
                    };
                    file_util::copy(exercise_file, target)?;
                }
            }

            // copy files directly in clone_path to dest
            for entry in WalkDir::new(clone_path)
                .into_iter()
                .filter_entry(|e| e.path().is_file())
            {
                let entry = entry?;
                let stripped = entry.path().strip_prefix(clone_path).unwrap();
                file_util::copy(entry.path(), dest.join(stripped))?;
            }
        }
        Plugin::Make(_) => {
            // make

            // copy src and test
            log::debug!("copying src and test");
            let main_path = clone_path.join("src");
            if main_path.exists() {
                file_util::copy(main_path, &dest)?;
            }
            let test_path = clone_path.join("test");
            if test_path.exists() {
                file_util::copy(test_path, &dest)?;
            }

            // copy files directly in tests to dest
            for entry in WalkDir::new(tests_dir).min_depth(1).max_depth(1) {
                let entry = entry?;
                if entry.path().is_file() {
                    file_util::copy(entry.path(), &dest)?;
                }
            }
        }
        _ => {
            // langs

            // copy libs
            log::debug!("copying lib");
            let lib_dir = clone_path.join("lib");
            if lib_dir.exists() {
                let lib_target = dest.join("lib");
                file_util::copy(lib_dir, lib_target)?;
            }

            // copy files from config
            log::debug!("copying files according to packaging config");
            let config = plugin.get_exercise_packaging_configuration(clone_path)?;
            for path in config.student_file_paths {
                let student_file = project_root.join(&path);
                if student_file.exists() {
                    file_util::copy(student_file, &dest)?;
                }
            }
            for path in config.exercise_file_paths {
                let exercise_file = tests_dir.join(&path);
                if exercise_file.exists() {
                    // todo --no-target-directory?
                    file_util::copy(exercise_file, &dest)?;
                }
            }

            // copy files directly in clone_path to dest
            log::debug!("copying files in clone path");
            for entry in WalkDir::new(clone_path).min_depth(1).max_depth(1) {
                let entry = entry?;
                if entry.path().is_file() {
                    file_util::copy(entry.path(), &dest)?;
                }
            }
        }
    }

    // copy extra student files
    // todo: necessary?
    log::debug!("copying extra student files");
    let tmc_project_yml = TmcProjectYml::from(clone_path)?;
    for extra_student_file in tmc_project_yml.extra_student_files {
        // todo secure path
        let source = received_dir.join(&extra_student_file);
        if source.exists() {
            let target = dest.join(&extra_student_file);
            file_util::copy(source, target)?;
        }
    }

    // make archive
    log::debug!("creating submission archive");
    let prefix = toplevel_dir_name
        .as_ref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new(""));
    let archive_file = file_util::create_file(target_path)?;
    if output_zip {
        let mut archive = ZipWriter::new(archive_file);
        for entry in WalkDir::new(&dest).into_iter().skip(1) {
            let entry = entry?;
            let entry_path = entry.path();
            let stripped = prefix.join(entry_path.strip_prefix(&dest).unwrap());
            log::debug!(
                "adding {} to zip at {}",
                entry_path.display(),
                stripped.display()
            );
            if entry_path.is_dir() {
                archive.add_directory_from_path(&stripped, FileOptions::default())?;
            } else {
                archive.start_file_from_path(&stripped, FileOptions::default())?;
                let mut file = file_util::open_file(entry_path)?;
                std::io::copy(&mut file, &mut archive).unwrap();
            }
        }
        archive.finish()?;
    } else {
        let mut archive = tar::Builder::new(archive_file);
        log::debug!(
            "appending \"{}\" at \"{}\"",
            dest.display(),
            prefix.display()
        );
        archive
            .append_dir_all(prefix, dest)
            .map_err(TmcError::TarAppend)?;
    }
    Ok(())
}

#[cfg(test)]
#[cfg(target_os = "linux")] // no maven plugin on other OS
mod test {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use walkdir::WalkDir;

    const MAVEN_CLONE: &str = "tests/data/MavenExercise";
    const MAVEN_ZIP: &str = "tests/data/MavenExercise.zip";

    const MAKE_CLONE: &str = "tests/data/MakeExercise";
    const MAKE_ZIP: &str = "tests/data/MakeExercise.zip";

    const PYTHON_CLONE: &str = "tests/data/PythonExercise";
    const PYTHON_ZIP: &str = "tests/data/PythonExercise.zip";

    fn init() {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "debug,j4rs=warn");
        }
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn generic_submission(clone: &str, zip: &str) -> (TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let output_archive = temp.path().join("output.tar");
        assert!(!output_archive.exists());

        let mut tmc_params = TmcParams::new();
        tmc_params.insert_string("param_one", "value_one").unwrap();
        tmc_params
            .insert_array("param_two", vec!["value_two", "value_three"])
            .unwrap();
        prepare_submission(
            Path::new(zip),
            &output_archive,
            None,
            tmc_params,
            Path::new(clone),
            None,
            false,
        )
        .unwrap();
        assert!(output_archive.exists());

        let output_file = file_util::open_file(&output_archive).unwrap();
        let mut archive = tar::Archive::new(output_file);
        let output_extracted = temp.path().join("output");
        archive.unpack(&output_extracted).unwrap();
        for entry in WalkDir::new(temp.path()) {
            log::debug!("{}", entry.unwrap().path().display());
        }
        (temp, output_extracted)
    }

    #[test]
    fn package_has_expected_files() {
        init();
        let (_temp, output) = generic_submission(MAVEN_CLONE, MAVEN_ZIP);
        // expected files
        assert!(output.join("src/main/java/SimpleStuff.java").exists());
        assert!(output.join("src/test/java/SimpleTest.java").exists());
        assert!(output.join("src/test/java/SimpleHiddenTest.java").exists());
        assert!(output.join("pom.xml").exists());
    }

    #[test]
    fn package_doesnt_have_unwanted_files() {
        init();
        let (_temp, output) = generic_submission(MAVEN_CLONE, MAVEN_ZIP);

        // files that should not be included
        assert!(!output.join("__MACOSX").exists());
        assert!(!output.join("desktop.ini").exists());
        assert!(!output.join("src/test/java/MadeUpTest.java").exists());
    }

    #[test]
    fn modified_test_file_not_included_in_package() {
        init();
        let (_temp, output) = generic_submission(MAVEN_CLONE, MAVEN_ZIP);

        // submission zip has a test file with the text MODIFIED...
        let zipfile = file_util::open_file(MAVEN_ZIP).unwrap();
        let mut zip = zip::ZipArchive::new(zipfile).unwrap();
        let mut modified = zip
            .by_name("MavenExercise/src/test/java/SimpleTest.java")
            .unwrap();
        let mut writer: Vec<u8> = vec![];
        std::io::copy(&mut modified, &mut writer).unwrap();
        let contents = String::from_utf8(writer).unwrap();
        assert!(contents.contains("MODIFIED"));
        // the text should not be in the package
        let test_file = fs::read_to_string(output.join("src/test/java/SimpleTest.java")).unwrap();
        assert!(!test_file.contains("MODIFIED"));
    }

    #[test]
    fn writes_tmcparams() {
        init();
        let (_temp, output) = generic_submission(MAVEN_CLONE, MAVEN_ZIP);

        let param_file = output.join(".tmcparams");
        assert!(param_file.exists());
        let conts = fs::read_to_string(param_file).unwrap();
        log::debug!("tmcparams {}", conts);
        let lines: Vec<_> = conts.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.contains(&"export param_one=value_one"));
        assert!(lines.contains(&"export param_two=( value_two value_three )"));
    }

    #[test]
    fn packages_with_toplevel_dir_name() {
        init();

        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("output.tar");

        assert!(!output.exists());
        prepare_submission(
            Path::new(MAVEN_ZIP),
            &output,
            Some("toplevel".to_string()),
            TmcParams::new(),
            Path::new(MAVEN_CLONE),
            None,
            false,
        )
        .unwrap();
        assert!(output.exists());

        let output = file_util::open_file(output).unwrap();
        let mut archive = tar::Archive::new(output);
        let output = temp.path().join("output");
        archive.unpack(&output).unwrap();
        for entry in WalkDir::new(temp.path()) {
            log::debug!("{}", entry.unwrap().path().display());
        }
        assert!(output
            .join("toplevel/src/test/java/SimpleHiddenTest.java")
            .exists());
        assert!(output.join("toplevel/pom.xml").exists());
    }

    #[test]
    fn packages_with_output_zip() {
        init();

        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("output.zip");

        assert!(!output.exists());
        prepare_submission(
            Path::new(MAVEN_ZIP),
            &output,
            None,
            TmcParams::new(),
            Path::new(MAVEN_CLONE),
            None,
            true,
        )
        .unwrap();
        assert!(output.exists());

        let output = file_util::open_file(output).unwrap();
        let mut archive = zip::ZipArchive::new(output).unwrap();
        archive
            .by_name("src/test/java/SimpleHiddenTest.java")
            .unwrap();
    }

    #[test]
    fn packages_with_toplevel_dir_and_output_zip() {
        init();

        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("output.zip");

        assert!(!output.exists());
        prepare_submission(
            Path::new(MAVEN_ZIP),
            &output,
            Some("toplevel".to_string()),
            TmcParams::new(),
            Path::new(MAVEN_CLONE),
            None,
            true,
        )
        .unwrap();
        assert!(output.exists());

        let output = file_util::open_file(output).unwrap();
        let mut archive = zip::ZipArchive::new(output).unwrap();
        archive
            .by_name("toplevel/src/test/java/SimpleHiddenTest.java")
            .unwrap();
        archive.by_name("toplevel/pom.xml").unwrap();
    }

    #[test]
    fn package_with_stub_tests() {
        init();

        let temp = tempfile::tempdir().unwrap();
        let output_arch = temp.path().join("output.tar");

        assert!(!output_arch.exists());
        prepare_submission(
            Path::new(MAVEN_ZIP),
            &output_arch,
            None,
            TmcParams::new(),
            Path::new(MAVEN_CLONE),
            Some(Path::new("tests/data/MavenStub.zip")),
            false,
        )
        .unwrap();
        assert!(output_arch.exists());

        let output_file = file_util::open_file(&output_arch).unwrap();
        let mut archive = tar::Archive::new(output_file);
        let output_extracted = temp.path().join("output");
        archive.unpack(&output_extracted).unwrap();
        for entry in WalkDir::new(temp.path()) {
            log::debug!("{}", entry.unwrap().path().display());
        }

        // visible tests included, hidden test isn't
        assert!(output_extracted
            .join("src/test/java/SimpleTest.java")
            .exists());
        assert!(!output_extracted
            .join("src/test/java/SimpleHiddenTest.java")
            .exists());
    }

    #[test]
    fn prepare_make_submission() {
        init();
        let (_temp, output) = generic_submission(MAKE_CLONE, MAKE_ZIP);

        // expected files
        assert!(output.join("src/main.c").exists());
        assert!(output.join("test/test_source.c").exists());
        assert!(output.join("Makefile").exists());
    }

    #[test]
    fn prepare_langs_submission() {
        init();
        let (_temp, output) = generic_submission(PYTHON_CLONE, PYTHON_ZIP);

        // expected files
        assert!(output.join("src/__main__.py").exists());
        assert!(output.join("test/test_greeter.py").exists());
        // assert!(output.join("tmc/points.py").exists()); // not included?
        assert!(output.join("__init__.py").exists());
    }
}
