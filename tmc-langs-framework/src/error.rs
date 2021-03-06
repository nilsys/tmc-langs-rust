use crate::io::tmc_zip;

use std::path::PathBuf;
use thiserror::Error;

// todo: make util error type and move variants there
#[derive(Error, Debug)]
pub enum TmcError {
    // IO
    #[error("Failed to open file at {0}")]
    OpenFile(PathBuf, #[source] std::io::Error),
    #[error("Failed to create file at {0}")]
    CreateFile(PathBuf, #[source] std::io::Error),
    #[error("Failed to remove file at {0}")]
    RemoveFile(PathBuf, #[source] std::io::Error),
    #[error("Failed to create dir(s) at {0}")]
    CreateDir(PathBuf, #[source] std::io::Error),
    #[error("Failed to remove dir at {0}")]
    RemoveDir(PathBuf, #[source] std::io::Error),
    #[error("Failed to create temporary directory")]
    TempDir(#[source] std::io::Error),
    #[error("Failed to rename {0} to {1}")]
    Rename(PathBuf, PathBuf, #[source] std::io::Error),
    #[error("Failed to write to {0}")]
    Write(PathBuf, #[source] std::io::Error),
    #[error("Failed to read zip archive at {0}")]
    ZipRead(PathBuf, #[source] std::io::Error),
    #[error("Error appending to tar")]
    TarAppend(#[source] std::io::Error),
    #[error("Error finishing tar")]
    TarFinish(#[source] std::io::Error),
    #[error("Failed to read line")]
    ReadLine(#[source] std::io::Error),
    #[error("Failed to copy file from {0} to {1}")]
    FileCopy(PathBuf, PathBuf, #[source] std::io::Error),
    #[error("Failed to open file at {0}")]
    FileOpen(PathBuf, #[source] std::io::Error),
    #[error("Failed to read file at {0}")]
    FileRead(PathBuf, #[source] std::io::Error),
    #[error("Failed to canonicalize path {0}")]
    Canonicalize(PathBuf, #[source] std::io::Error),
    #[error("Error occurred in a child process")]
    Process(#[source] std::io::Error),
    #[error("Failed to set permissions for {0}")]
    SetPermissions(PathBuf, #[source] std::io::Error),
    #[error("Invalid parameter value: {0}")]
    InvalidParam(String),
    #[error("File {0} not in given project root {1}")]
    FileNotInProject(PathBuf, PathBuf),
    #[error("Path {0} is not absolute")]
    PathNotAbsolute(PathBuf),

    #[error("Path {0} contained invalid UTF8")]
    UTF8(PathBuf),
    #[error("Path {0} contained no file name")]
    NoFileName(PathBuf),

    #[error("No matching plugin found for {0}")]
    PluginNotFound(PathBuf),
    #[error("No project directory found in archive during unzip")]
    NoProjectDirInZip,
    #[error("Running command '{0}' failed")]
    CommandFailed(&'static str, #[source] std::io::Error),

    #[error("Failed to spawn command: {0}")]
    CommandSpawn(&'static str, #[source] std::io::Error),

    #[error("Error in plugin")]
    Plugin(#[from] Box<dyn std::error::Error + 'static + Send + Sync>),

    #[error(transparent)]
    YamlDeserialization(#[from] serde_yaml::Error),
    #[error(transparent)]
    ZipError(#[from] tmc_zip::ZipError),
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),

    #[error("Command not found")]
    CommandNotFound(#[from] CommandNotFound),
}

// == Collection of errors likely to be useful in multiple plugins which can be special cased without needing a plugin's specific error type ==
#[derive(Error, Debug)]
#[error("The executable for \"{name}\" could not be found ({path}). Please make sure you have installed it correctly.")]
pub struct CommandNotFound {
    pub name: &'static str,
    pub path: PathBuf,
    pub source: std::io::Error,
}
