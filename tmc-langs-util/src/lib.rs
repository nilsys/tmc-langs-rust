//! Contains the task executor

pub mod task_executor;

pub use tmc_langs_abstraction::{Strategy, ValidationResult};
pub use tmc_langs_framework::{
    domain::{ExerciseDesc, ExercisePackagingConfiguration, RunResult, RunStatus},
    plugin::Language,
    TmcError,
};
