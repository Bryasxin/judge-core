use crate::handler::Handler;
use futures::future::join_all;
use shared::rpc::{JudgeResult, Submission};

#[derive(Debug, Clone, Copy)]
pub struct Engine;

impl Engine {
    pub async fn judge(
        handler: impl Handler,
        submission: Submission,
        compile_time_limit_ms: u64,
    ) -> JudgeResult {
        let need_compile = handler.needs_compile();

        let ctx = match handler.prepare(&submission.source_code).await {
            Ok(info) => info,
            Err(err) => {
                return JudgeResult::InternalError {
                    error_message: err.to_string(),
                };
            }
        };

        if need_compile {
            let compile_info = match handler.compile(&ctx, compile_time_limit_ms).await {
                Ok(info) => info.unwrap(),
                Err(err) => return err.into(),
            };

            if !compile_info.status_code.success() {
                let message = format!(
                    "Stdout:\n{}\nStderr:\n{}",
                    compile_info.stdout, compile_info.stderr
                );

                return JudgeResult::CompilationError {
                    compiler_message: message,
                };
            }
        }

        let results = join_all(submission.test_cases.iter().map(|case| {
            // stdout for judging: max 2x expected output size
            let stdout_limit_bytes = case.expected_output.len() * 2;
            // stderr for debugging: allow up to 128 KiB
            let stderr_limit_bytes = 128 * 1024;
            handler.execute(
                &ctx,
                &case.input_data,
                submission.limits.time_ms,
                submission.limits.memory_kib,
                stdout_limit_bytes,
                stderr_limit_bytes,
            )
        }))
        .await;

        // Collect resource usage statistics
        let mut max_cpu_time_ms = 0u64;
        let mut max_real_time_ms = 0u64;
        let mut max_memory_kib = 0u64;

        for (idx, result) in results.into_iter().enumerate() {
            match result {
                Ok(result) => {
                    // Check time
                    if result.resource_usage.cpu_time_ms > submission.limits.time_ms {
                        return JudgeResult::TimeLimitExceeded;
                    }

                    // Check memory
                    if result.resource_usage.memory_kib > submission.limits.memory_kib {
                        return JudgeResult::MemoryLimitExceeded;
                    }

                    // Check exit code
                    if !result.status_code.success() {
                        let output_formated =
                            format!("Stdout:\n{}\nStderr:\n{}", result.stdout, result.stderr);

                        return JudgeResult::RuntimeError {
                            actual_output: output_formated,
                            error_message: "Non-zero exit code".into(),
                        };
                    }

                    // Check output
                    let expected = submission.test_cases[idx].expected_output.trim();
                    let actual = result.stdout.trim();

                    if expected != actual {
                        return JudgeResult::WrongAnswer {
                            expected_output: expected.to_string(),
                            actual_output: actual.to_string(),
                        };
                    }

                    // Update maximum resource usage
                    max_cpu_time_ms = max_cpu_time_ms.max(result.resource_usage.cpu_time_ms);
                    max_real_time_ms = max_real_time_ms.max(result.resource_usage.real_time_ms);
                    max_memory_kib = max_memory_kib.max(result.resource_usage.memory_kib);
                }
                Err(err) => return err.into(),
            }
        }

        if let Err(e) = handler.cleanup(&ctx).await {
            return e.into();
        }

        JudgeResult::Accepted {
            cpu_time_ms: max_cpu_time_ms,
            real_time_ms: max_real_time_ms,
            memory_kib: max_memory_kib,
        }
    }
}
