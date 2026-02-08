#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum JudgeResult {
    Accepted {
        cpu_time_ms: u64,
        real_time_ms: u64,
        memory_kib: u64,
    },
    WrongAnswer {
        expected_output: String,
        actual_output: String,
    },
    RuntimeError {
        actual_output: String,
        error_message: String,
    },
    CompilationError {
        compiler_message: String,
    },
    InternalError {
        error_message: String,
    },
    TimeLimitExceeded,
    MemoryLimitExceeded,
    OutputLimitExceeded,
    PresentationError,
}

/// Judge request
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JudgeRequest {
    /// RPC Id
    pub id: usize,

    /// Language
    pub language: Language,

    /// Source code
    pub source_code: String,

    /// Test cases
    pub test_cases: Vec<TestCase>,

    /// Resource limits
    pub limits: ResourceLimits,
}

/// Available languages
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Language {
    Cpp,
}

impl JudgeResult {
    /// Convert into judge response, is_fatal_error if internal error
    pub fn into_judge_response(self, id: usize) -> JudgeResponse {
        let is_fatal = matches!(self, Self::InternalError { error_message: _ });

        JudgeResponse {
            id,
            is_fatal_error: Some(is_fatal),
            result: self,
        }
    }
}

/// Judge response
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JudgeResponse {
    /// RPC Id
    pub id: usize,

    /// If true, this error is fatal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_fatal_error: Option<bool>,

    /// Judge result type
    pub result: JudgeResult,
}

/// Resource limits
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ResourceLimits {
    /// Time limit in milliseconds
    pub time_ms: u64,

    /// Memory limit in KiB
    pub memory_kib: u64,
}

/// Test case
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TestCase {
    /// Input data
    pub input_data: String,

    /// Expected output
    pub expected_output: String,
}
