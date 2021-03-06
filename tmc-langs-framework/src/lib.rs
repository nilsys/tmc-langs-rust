//! Contains functionality for dealing with projects.

pub mod command;
pub mod domain;
pub mod error;
pub mod io;
pub mod plugin;
pub mod policy;

pub use error::TmcError;
pub use plugin::LanguagePlugin;
pub use policy::StudentFilePolicy;
pub use zip;

use domain::TmcProjectYml;

pub type Result<T> = std::result::Result<T, TmcError>;
