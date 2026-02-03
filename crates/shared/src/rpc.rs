#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum JudgeResult {
    // TODO: Add more monitor fields like `cpu_time_ms`, `memory_kib`
    Accepted {
        cpu_time_ms: u64,
        real_time_ms: u64,
        memory_kib: u64,
    },
    WrongAnswer {
        expected_output: String,
        actual_output: String,
    },
    TimeLimitExceeded,
    MemoryLimitExceeded,
    RuntimeError {
        actual_output: String,
        error_message: String,
    },
    CompilationError {
        compiler_message: String,
    },
    // HACK: It's hard to detect presentation error, so we just ignore it
    PresentationError,
    // HACK: This implementation is not detectable
    OutputLimitExceeded,
    InternalError {
        error_message: String,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Language {
    Cpp,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Submission {
    pub language: Language,
    pub source_code: String,
    pub test_cases: Vec<TestCase>,
    pub limits: ResourceLimits,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ResourceLimits {
    pub time_ms: u64,
    pub memory_kib: u64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TestCase {
    pub input_data: String,
    pub expected_output: String,
}
