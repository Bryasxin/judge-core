use crate::handler::Handler;
use shared::rpc::{JudgeRequest, JudgeResponse, JudgeResult};

#[derive(Debug, Clone, Copy)]
pub struct Engine;

impl Engine {
    pub async fn judge(
        handler: impl Handler,
        request: JudgeRequest,
        compile_time_limit_ms: u64,
    ) -> JudgeResponse {
        let request_id = request.id;
        let need_compile = handler.needs_compile();

        let ctx = match handler.prepare(&request.source_code).await {
            Ok(info) => info,
            Err(err) => {
                let err: JudgeResult = err.into();
                return err.into_judge_response(request_id);
            }
        };

        if need_compile {
            let compile_info = match handler.compile(&ctx, compile_time_limit_ms).await {
                Ok(info) => info.unwrap(),
                Err(err) => {
                    let err: JudgeResult = err.into();
                    return err.into_judge_response(request_id);
                }
            };

            if !compile_info.status_code.success() {
                let message = format!(
                    "Stdout:\n{}\nStderr:\n{}",
                    compile_info.stdout, compile_info.stderr
                );

                return JudgeResult::CompilationError {
                    compiler_message: message,
                }
                .into_judge_response(request_id);
            }
        }

        let mut max_cpu_time_ms = 0u64;
        let mut max_real_time_ms = 0u64;
        let mut max_memory_kib = 0u64;

        for case in &request.test_cases {
            let stdout_limit_bytes = case.expected_output.len() * 2;
            let stderr_limit_bytes = 128 * 1024;
            let result = match handler
                .execute(
                    &ctx,
                    &case.input_data,
                    request.limits.time_ms,
                    request.limits.memory_kib,
                    stdout_limit_bytes,
                    stderr_limit_bytes,
                )
                .await
            {
                Ok(result) => result,
                Err(err) => {
                    let err: JudgeResult = err.into();
                    return err.into_judge_response(request_id);
                }
            };

            // Check time
            if result.resource_usage.cpu_time_ms > request.limits.time_ms {
                return JudgeResult::TimeLimitExceeded.into_judge_response(request_id);
            }

            // Check memory
            if result.resource_usage.memory_kib > request.limits.memory_kib {
                return JudgeResult::MemoryLimitExceeded.into_judge_response(request_id);
            }

            // Check exit code
            if !result.status_code.success() {
                let output_formated =
                    format!("Stdout:\n{}\nStderr:\n{}", result.stdout, result.stderr);

                return JudgeResult::RuntimeError {
                    actual_output: output_formated,
                    error_message: "Non-zero exit code".into(),
                }
                .into_judge_response(request_id);
            }

            // Check output
            let expected = case.expected_output.trim();
            let actual = result.stdout.trim();

            if expected != actual {
                return JudgeResult::WrongAnswer {
                    expected_output: expected.to_string(),
                    actual_output: actual.to_string(),
                }
                .into_judge_response(request_id);
            }

            // Update maximum resource usage
            max_cpu_time_ms = max_cpu_time_ms.max(result.resource_usage.cpu_time_ms);
            max_real_time_ms = max_real_time_ms.max(result.resource_usage.real_time_ms);
            max_memory_kib = max_memory_kib.max(result.resource_usage.memory_kib);
        }

        if let Err(e) = handler.cleanup(&ctx).await {
            let e: JudgeResult = e.into();
            return e.into_judge_response(request_id);
        }

        JudgeResult::Accepted {
            cpu_time_ms: max_cpu_time_ms,
            real_time_ms: max_real_time_ms,
            memory_kib: max_memory_kib,
        }
        .into_judge_response(request_id)
    }
}
