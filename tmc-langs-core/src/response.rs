//! Contains types which model the JSON responses from tmc-server

use crate::CoreError;

use lazy_static::lazy_static;
use regex::Regex;
use schemars::JsonSchema;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use tmc_langs_util::ValidationResult;

/// Models the responses from tmc-server, which can either
/// be some successful response, a single error or a list of errors
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Ok(T),
    Errs(ResponseErrors),
    Err(ResponseError),
}

impl<T> Response<T> {
    /// Convenience function to easily propagate error responses
    pub fn into_result(self) -> Result<T, CoreError> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(err) => Err(err.into()),
            Self::Errs(errs) => Err(errs.into()),
        }
    }
}

/// Represents an error response from tmc-server
#[derive(Debug, Error, Deserialize)]
#[error("Response contained errors: {errors:#?}")]
#[serde(deny_unknown_fields)] // prevents responses with an errors field from being parsed as an error
pub struct ResponseErrors {
    pub errors: Vec<String>,
}

/// Represents an error response from tmc-server
#[derive(Debug, Error, Deserialize)]
#[error("Response contained an error: {error:#?}. Obsolete client: {}", obsolete_client.unwrap_or_default())]
#[serde(deny_unknown_fields)] // prevents responses with an error field from being parsed as an error
pub struct ResponseError {
    pub error: String,
    pub obsolete_client: Option<bool>,
}

/// OAuth2 credentials
#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub application_id: String,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub email: String,
    pub administrator: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Organization {
    pub name: String,
    pub information: String,
    pub slug: String,
    pub logo_path: String,
    pub pinned: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Course {
    pub id: usize,
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub details_url: String,
    pub unlock_url: String,
    pub reviews_url: String,
    pub comet_url: String,
    pub spyware_urls: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CourseData {
    pub name: String,
    pub hide_after: Option<String>,
    pub hidden: bool,
    pub cache_version: Option<usize>,
    pub spreadsheet_key: Option<String>,
    pub hidden_if_registered_after: Option<String>,
    pub refreshed_at: Option<String>,
    pub locked_exercise_points_visible: bool,
    pub description: Option<String>,
    pub paste_visibility: Option<String>,
    pub formal_name: Option<String>,
    pub certificate_downloadable: Option<bool>,
    pub certificate_unlock_spec: Option<String>,
    pub organization_id: Option<usize>,
    pub disabled_status: Option<String>,
    pub title: Option<String>,
    pub material_url: Option<String>,
    pub course_template_id: Option<usize>,
    pub hide_submission_results: bool,
    pub external_scoreboard_url: Option<String>,
    pub organization_slug: Option<String>,
}

/// Represents a course details response from tmc-server,
/// converted to the more convenient CourseDetails during deserialization
#[derive(Debug, Deserialize)]
struct CourseDetailsWrapper {
    pub course: CourseDetailsInner,
}

// TODO: improve
#[derive(Debug, Deserialize)]
struct CourseDetailsInner {
    #[serde(flatten)]
    pub course: Course,
    pub unlockables: Vec<String>,
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(from = "CourseDetailsWrapper")]
pub struct CourseDetails {
    #[serde(flatten)]
    pub course: Course,
    pub unlockables: Vec<String>,
    pub exercises: Vec<Exercise>,
}

impl From<CourseDetailsWrapper> for CourseDetails {
    fn from(value: CourseDetailsWrapper) -> Self {
        Self {
            course: value.course.course,
            unlockables: value.course.unlockables,
            exercises: value.course.exercises,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Exercise {
    pub id: usize,
    pub name: String,
    pub locked: bool,
    pub deadline_description: Option<String>,
    pub deadline: Option<String>,
    pub soft_deadline: Option<String>,
    pub soft_deadline_description: Option<String>,
    pub checksum: String,
    pub return_url: String,
    pub zip_url: String,
    pub returnable: bool,
    pub requires_review: bool,
    pub attempted: bool,
    pub completed: bool,
    pub reviewed: bool,
    pub all_review_points_given: bool,
    pub memory_limit: Option<usize>,
    pub runtime_params: Vec<String>,
    pub valgrind_strategy: String,
    pub code_review_requests_enabled: bool,
    pub run_tests_locally_action_enabled: bool,
    pub latest_submission_url: Option<String>,
    pub latest_submission_id: Option<usize>,
    pub solution_zip_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CourseExercise {
    pub id: usize,
    pub available_points: Vec<ExercisePoint>,
    pub awarded_points: Vec<String>,
    pub name: String,
    pub publish_time: Option<String>,
    pub solution_visible_after: Option<String>,
    pub deadline: Option<String>,
    pub soft_deadline: Option<String>,
    pub disabled: bool,
    pub unlocked: bool,
}

#[derive(Debug, Deserialize)]
pub struct CourseDataExercise {
    pub id: usize,
    pub available_points: Vec<ExercisePoint>,
    pub name: String,
    pub publish_time: Option<String>,
    pub solution_visible_after: Option<String>,
    pub deadline: Option<String>,
    pub disabled: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExercisePoint {
    pub id: usize,
    pub exercise_id: usize,
    pub name: String,
    pub requires_review: bool,
}

#[derive(Debug, Deserialize)]
pub struct CourseDataExercisePoint {
    awarded_point: AwardedPoint,
    exercise_id: usize,
}

#[derive(Debug, Deserialize)]
pub struct AwardedPoint {
    id: usize,
    course_id: usize,
    user_id: usize,
    submission_id: usize,
    name: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExerciseDetails {
    pub course_name: String,
    pub course_id: usize,
    pub code_review_requests_enabled: bool,
    pub run_tests_locally_action_enabled: bool,
    pub exercise_name: String,
    pub exercise_id: usize,
    pub unlocked_at: Option<String>,
    pub deadline: Option<String>,
    pub submissions: Vec<ExerciseSubmission>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Submission {
    pub id: usize,
    pub user_id: usize,
    pub pretest_error: Option<String>,
    pub created_at: String,
    pub exercise_name: String,
    pub course_id: usize,
    pub processed: bool,
    pub all_tests_passed: bool,
    pub points: Option<String>,
    pub processing_tried_at: Option<String>,
    pub processing_began_at: Option<String>,
    pub processing_completed_at: Option<String>,
    pub times_sent_to_sandbox: usize,
    pub processing_attempts_started_at: String,
    pub params_json: Option<String>,
    pub requires_review: bool,
    pub requests_review: bool,
    pub reviewed: bool,
    pub message_for_reviewer: String,
    pub newer_submission_reviewed: bool,
    pub review_dismissed: bool,
    pub paste_available: bool,
    pub message_for_paste: String,
    pub paste_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExerciseSubmission {
    pub exercise_name: String,
    pub id: usize,
    pub user_id: usize,
    pub course_id: usize,
    pub created_at: String,
    pub all_tests_passed: bool,
    pub points: Option<String>,
    pub submitted_zip_url: String,
    pub paste_url: Option<String>,
    pub processing_time: Option<usize>,
    pub reviewed: bool,
    pub requests_review: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct NewSubmission {
    pub show_submission_url: String,
    pub paste_url: String, // use Option and serde_with::string_empty_as_none ?
    pub submission_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)] // TODO: tag
pub enum SubmissionProcessingStatus {
    Processing(SubmissionProcessing),
    Finished(Box<SubmissionFinished>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmissionProcessing {
    pub status: SubmissionStatus,
    pub sandbox_status: SandboxStatus,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    Created,
    SendingToSandbox,
    ProcessingOnSandbox,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct SubmissionFinished {
    pub api_version: usize,
    pub all_tests_passed: Option<bool>,
    pub user_id: usize,
    pub login: String,
    pub course: String,
    pub exercise_name: String,
    pub status: SubmissionStatus,
    pub points: Vec<String>,
    pub valgrind: Option<String>,
    pub submission_url: String,
    pub solution_url: Option<String>,
    pub submitted_at: String,
    pub processing_time: Option<usize>,
    pub reviewed: bool,
    pub requests_review: bool,
    pub paste_url: Option<String>,
    pub message_for_paste: Option<String>,
    pub missing_review_points: Vec<String>,
    pub test_cases: Option<Vec<TestCase>>,
    pub feedback_questions: Option<Vec<SubmissionFeedbackQuestion>>,
    pub feedback_answer_url: Option<String>,
    pub error: Option<String>,
    pub validations: Option<ValidationResult>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionStatus {
    Processing,
    Fail,
    Ok,
    Error,
    Hidden,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SubmissionFeedbackResponse {
    pub api_version: usize,
    pub status: SubmissionStatus,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct TestCase {
    pub name: String,
    pub successful: bool,
    pub message: Option<String>,
    pub exception: Option<Vec<String>>,
    pub detailed_message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct SubmissionFeedbackQuestion {
    pub id: usize,
    pub question: String,
    pub kind: SubmissionFeedbackKind,
}

#[derive(Debug, PartialEq, Eq, JsonSchema)]
pub enum SubmissionFeedbackKind {
    Text,
    IntRange { lower: usize, upper: usize },
}

impl<'de> Deserialize<'de> for SubmissionFeedbackKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(SubmissionFeedbackKindVisitor {})
    }
}

impl Serialize for SubmissionFeedbackKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Self::Text => "text".to_string(),
            Self::IntRange { lower, upper } => format!("intrange[{}..{}]", lower, upper),
        };
        serializer.serialize_str(&s)
    }
}

struct SubmissionFeedbackKindVisitor {}

// parses "text" into Text, and "intrange[x..y]" into IntRange {lower: x, upper: y}
impl<'de> Visitor<'de> for SubmissionFeedbackKindVisitor {
    type Value = SubmissionFeedbackKind;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("\"text\" or \"intrange[x..y]\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref RANGE: Regex = Regex::new(r#"intrange\[(\d+)\.\.(\d+)\]"#).unwrap();
        }
        if value == "text" {
            Ok(SubmissionFeedbackKind::Text)
        } else if let Some(captures) = RANGE.captures(&value) {
            let lower = &captures[1];
            let lower = usize::from_str(lower).map_err(|e| {
                E::custom(format!(
                    "error parsing intrange lower bound {}: {}",
                    lower, e
                ))
            })?;
            let upper = &captures[2];
            let upper = usize::from_str(upper).map_err(|e| {
                E::custom(format!(
                    "error parsing intrange upper bound {}: {}",
                    upper, e
                ))
            })?;
            Ok(SubmissionFeedbackKind::IntRange { lower, upper })
        } else {
            Err(E::custom("expected \"text\" or \"intrange[x..y]\""))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Review {
    pub submission_id: String,
    pub exercise_name: String,
    pub id: usize,
    pub marked_as_read: bool,
    pub reviewer_name: String,
    pub review_body: String,
    pub points: Vec<String>,
    pub points_not_awarded: Vec<String>,
    pub url: String,
    pub update_url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UpdateResult {
    pub created: Vec<Exercise>,
    pub updated: Vec<Exercise>,
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn course_details_de() {
        init();

        let details = serde_json::json!(
            {
                "course": {
                    "comet_url": "c",
                    "description": "d",
                    "details_url": "du",
                    "id": 1,
                    "name": "n",
                    "reviews_url": "r",
                    "spyware_urls": [
                        "s"
                    ],
                    "title": "t",
                    "unlock_url": "u",
                    "unlockables": ["a"],
                    "exercises": []
                }
            }
        );
        assert!(serde_json::from_value::<CourseDetails>(details).is_ok());
    }

    #[test]
    fn feedback_kind_de() {
        init();

        let text = serde_json::json!("text");
        let text: SubmissionFeedbackKind = serde_json::from_value(text).unwrap();
        if let SubmissionFeedbackKind::Text = text {
        } else {
            panic!("wrong type")
        }

        let intrange = serde_json::json!("intrange[1..5]");
        let intrange: SubmissionFeedbackKind = serde_json::from_value(intrange).unwrap();
        if let SubmissionFeedbackKind::IntRange { lower: 1, upper: 5 } = intrange {
        } else {
            panic!("wrong type")
        }
    }

    #[test]
    fn feedback_kind_se() {
        init();
        use serde_json::Value;

        let text = SubmissionFeedbackKind::Text;
        let text = serde_json::to_value(&text).unwrap();
        assert_eq!(text, Value::String("text".to_string()));

        let range = SubmissionFeedbackKind::IntRange { lower: 1, upper: 5 };
        let range = serde_json::to_value(&range).unwrap();
        assert_eq!(range, Value::String("intrange[1..5]".to_string()));
    }

    #[test]
    fn deserializes_struct_with_error_field() {
        let json = r#"{
  "api_version": 7,
  "all_tests_passed": false,
  "user_id": 123,
  "login": "log",
  "course": "cou",
  "exercise_name": "exe",
  "status": "error",
  "points": [],
  "validations": null,
  "valgrind": null,
  "submission_url": "sub",
  "solution_url": "sol",
  "submitted_at": "sat",
  "processing_time": null,
  "reviewed": false,
  "requests_review": false,
  "paste_url": null,
  "message_for_paste": null,
  "missing_review_points": [],
  "error": "error msg"
}"#;
        let s: Response<SubmissionProcessingStatus> = serde_json::from_str(json).unwrap();
        if let Response::Ok(_) = s {
        } else {
            panic!("parse failed")
        }
    }
}
