#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum JudgeResult {
    Accepted {
        id: String,
        cpu_time_ms: u64,
        real_time_ms: u64,
        memory_kib: u64,
    },
    WrongAnswer {
        id: String,
        expected_output: String,
        actual_output: String,
    },
    TimeLimitExceeded {
        id: String,
    },
    MemoryLimitExceeded {
        id: String,
    },
    RuntimeError {
        id: String,
        actual_output: String,
        error_message: String,
    },
    CompilationError {
        id: String,
        compiler_message: String,
    },
    PresentationError {
        id: String,
    },
    OutputLimitExceeded {
        id: String,
    },
    InternalError {
        id: String,
        error_message: String,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Language {
    Cpp,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Submission {
    pub id: String,
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
