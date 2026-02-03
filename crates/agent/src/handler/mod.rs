mod cpp;
pub use cpp::CppHandler;

use shared::rpc::JudgeResult;
use std::{path::PathBuf, process::ExitStatus};

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Time limit exceeded")]
    TimeLimitExceeded,
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,
    #[error("Output limit exceeded")]
    OutputLimitExceeded,
    #[error("Internal error: {0}")]
    InternalError(&'static str),
    #[error("Cgroup error: {0}")]
    CgroupError(#[from] cgroups_rs::fs::error::Error),
    #[error("Parse cpu stats error: {0}")]
    ParseCpuStatsError(#[from] crate::utils::ParseCpuStatsError),
}

impl From<HandlerError> for JudgeResult {
    fn from(val: HandlerError) -> Self {
        match val {
            HandlerError::IoError(err) => JudgeResult::InternalError {
                error_message: err.to_string(),
            },
            HandlerError::TimeLimitExceeded => JudgeResult::TimeLimitExceeded,
            HandlerError::MemoryLimitExceeded => JudgeResult::MemoryLimitExceeded,
            HandlerError::OutputLimitExceeded => JudgeResult::OutputLimitExceeded,
            HandlerError::InternalError(err) => JudgeResult::InternalError {
                error_message: err.into(),
            },
            HandlerError::CgroupError(e) => JudgeResult::InternalError {
                error_message: e.to_string(),
            },
            HandlerError::ParseCpuStatsError(e) => JudgeResult::InternalError {
                error_message: e.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub work_dir: PathBuf,
    pub source_file: PathBuf,
    pub executable_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CompileInfo {
    pub status_code: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct ExecuteInfo {
    pub status_code: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_kib: u64,
    pub real_time_ms: u64,
    pub cpu_time_ms: u64,
}

/// Language related handler
pub trait Handler {
    /// Whether the handler needs compilation
    ///
    /// If false, [`Handler::compile`] will not be called
    fn needs_compile(&self) -> bool;

    /// Prepare the environment for compilation
    async fn prepare(&self, source_code: &str) -> Result<ExecutionContext, HandlerError>;

    /// Compile the source code
    ///
    /// Return [`Option::None`] if compilation is not needed
    async fn compile(
        &self,
        context: &ExecutionContext,
        time_limit_ms: u64,
    ) -> Result<Option<CompileInfo>, HandlerError>;

    /// Execute the compiled program once
    ///
    /// Handler should handle time limit and memory limit
    ///
    /// Note: stderr is for debugging (user), stdout is for judging (expected output comparison)
    async fn execute(
        &self,
        context: &ExecutionContext,
        input_data: &str,
        time_limit_ms: u64,
        memory_limit_kib: u64,
        stdout_limit_bytes: usize,
        stderr_limit_bytes: usize,
    ) -> Result<ExecuteInfo, HandlerError>;

    /// Cleanup the environment
    ///
    /// REVIEW: Should we use it? The agent will only be executed once and then the MicroVM will be destroyed
    async fn cleanup(&self, context: &ExecutionContext) -> Result<(), HandlerError>;
}
