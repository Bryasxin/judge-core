use crate::handler::Handler;
use shared::rpc::{JudgeResult, Submission};

#[derive(Debug, Clone, Copy)]
pub struct Engine;

impl Engine {
    pub async fn judge(
        handler: impl Handler,
        submission: Submission,
        compile_time_limit_ms: u64,
    ) -> JudgeResult {
        let submission_id = submission.id;
        let need_compile = handler.needs_compile();

        let ctx = match handler.prepare(&submission.source_code).await {
            Ok(info) => info,
            Err(err) => {
                return err.into_judge_result(&submission_id);
            }
        };

        if need_compile {
            let compile_info = match handler.compile(&ctx, compile_time_limit_ms).await {
                Ok(info) => info.unwrap(),
                Err(err) => return err.into_judge_result(&submission_id),
            };

            if !compile_info.status_code.success() {
                let message = format!(
                    "Stdout:\n{}\nStderr:\n{}",
                    compile_info.stdout, compile_info.stderr
                );

                return JudgeResult::CompilationError {
                    id: submission_id,
                    compiler_message: message,
                };
            }
        }

        let mut max_cpu_time_ms = 0u64;
        let mut max_real_time_ms = 0u64;
        let mut max_memory_kib = 0u64;

        for case in &submission.test_cases {
            let stdout_limit_bytes = case.expected_output.len() * 2;
            let stderr_limit_bytes = 128 * 1024;
            let result = match handler
                .execute(
                    &ctx,
                    &case.input_data,
                    submission.limits.time_ms,
                    submission.limits.memory_kib,
                    stdout_limit_bytes,
                    stderr_limit_bytes,
                )
                .await
            {
                Ok(result) => result,
                Err(err) => return err.into_judge_result(&submission_id),
            };

            // Check time
            if result.resource_usage.cpu_time_ms > submission.limits.time_ms {
                return JudgeResult::TimeLimitExceeded { id: submission_id };
            }

            // Check memory
            if result.resource_usage.memory_kib > submission.limits.memory_kib {
                return JudgeResult::MemoryLimitExceeded { id: submission_id };
            }

            // Check exit code
            if !result.status_code.success() {
                let output_formated =
                    format!("Stdout:\n{}\nStderr:\n{}", result.stdout, result.stderr);

                return JudgeResult::RuntimeError {
                    id: submission_id,
                    actual_output: output_formated,
                    error_message: "Non-zero exit code".into(),
                };
            }

            // Check output
            let expected = case.expected_output.trim();
            let actual = result.stdout.trim();

            if expected != actual {
                return JudgeResult::WrongAnswer {
                    id: submission_id,
                    expected_output: expected.to_string(),
                    actual_output: actual.to_string(),
                };
            }

            // Update maximum resource usage
            max_cpu_time_ms = max_cpu_time_ms.max(result.resource_usage.cpu_time_ms);
            max_real_time_ms = max_real_time_ms.max(result.resource_usage.real_time_ms);
            max_memory_kib = max_memory_kib.max(result.resource_usage.memory_kib);
        }

        if let Err(e) = handler.cleanup(&ctx).await {
            return e.into_judge_result(&submission_id);
        }

        JudgeResult::Accepted {
            id: submission_id,
            cpu_time_ms: max_cpu_time_ms,
            real_time_ms: max_real_time_ms,
            memory_kib: max_memory_kib,
        }
    }
}
