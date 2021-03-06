//! Language plugin for C#
mod error;
mod plugin;

pub use error::CSharpError;
pub use plugin::{CSharpPlugin, CSharpStudentFilePolicy};
use tmc_langs_framework::domain::TestResult;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CSTestResult {
    name: String,
    passed: bool,
    message: String,
    points: Vec<String>,
    error_stack_trace: Vec<String>,
}

impl From<CSTestResult> for TestResult {
    fn from(test_result: CSTestResult) -> Self {
        TestResult {
            name: test_result.name,
            successful: test_result.passed,
            message: test_result.message,
            exception: test_result.error_stack_trace,
            points: test_result.points,
        }
    }
}
