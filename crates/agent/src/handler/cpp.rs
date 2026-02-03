use crate::{
    handler::{ExecutionContext, Handler, HandlerError},
    utils::CpuStats,
};
use cgroups_rs::{
    CgroupPid,
    fs::{cgroup_builder::CgroupBuilder, cpu::CpuController, hierarchies, memory::MemController},
};
use std::{
    process::{Output, Stdio},
    str::FromStr,
    time::Duration,
};
use tempfile::tempdir;
use tokio::{
    fs::{remove_dir, remove_file},
    io::AsyncWriteExt,
    process::Command,
    time::{Instant, timeout},
};

#[derive(Debug, Clone, Copy)]
pub struct CppHandler;

impl Handler for CppHandler {
    fn needs_compile(&self) -> bool {
        true
    }

    async fn prepare(
        &self,
        source_code: &str,
    ) -> Result<super::ExecutionContext, super::HandlerError> {
        let temp_dir = tempdir()?.keep();
        let source_code_path = temp_dir.join("input.cpp");
        let executable_path = temp_dir.join("output.executable");

        tokio::fs::File::create_new(&source_code_path)
            .await?
            .write_all(source_code.as_bytes())
            .await?;

        Ok(ExecutionContext {
            work_dir: temp_dir,
            source_file: source_code_path,
            executable_file: executable_path,
        })
    }

    async fn compile(
        &self,
        context: &super::ExecutionContext,
        time_limit_ms: u64,
    ) -> Result<Option<super::CompileInfo>, super::HandlerError> {
        // Using o2 optimization level and suppressing warnings
        let cmd = Command::new("g++")
            .arg("-w")
            .arg("-O2")
            .arg(&context.source_file)
            .arg("-o")
            .arg(&context.executable_file)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = match timeout(Duration::from_millis(time_limit_ms), async move {
            Ok::<Output, std::io::Error>(cmd.wait_with_output().await?)
        })
        .await
        {
            Err(_) => return Err(HandlerError::TimeLimitExceeded),
            Ok(Err(e)) => return Err(e.into()),
            Ok(Ok(output)) => output,
        };

        Ok(Some(super::CompileInfo {
            status_code: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        }))
    }

    async fn execute(
        &self,
        context: &super::ExecutionContext,
        input_data: &str,
        time_limit_ms: u64,
        memory_limit_kib: u64,
        output_limit_u8: usize,
    ) -> Result<super::ExecuteInfo, super::HandlerError> {
        let now = Instant::now();

        let mut cmd = Command::new(&context.executable_file)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let pid = cmd
            .id()
            .ok_or(HandlerError::InternalError("Cannot get child process pid"))?;

        // Create cgroup to gather resource usage
        let hier = hierarchies::auto();
        let cg = CgroupBuilder::new(&format!("judge-cpp-execute-{}", pid))
            .cpu()
            .done()
            .memory()
            .done()
            .build(hier)?;
        cg.add_task(CgroupPid::from(pid as u64))?;
        let memory_controller: &MemController = cg.controller_of().unwrap();
        let cpu_controller: &CpuController = cg.controller_of().unwrap();

        // It is impossible to fail
        let mut stdin = cmd.stdin.take().unwrap();
        stdin.write_all(input_data.as_bytes()).await?;
        drop(stdin);

        let output = match timeout(Duration::from_millis(time_limit_ms), async move {
            Ok::<Output, std::io::Error>(cmd.wait_with_output().await?)
        })
        .await
        {
            Err(_) => return Err(HandlerError::TimeLimitExceeded),
            Ok(Err(e)) => return Err(e.into()),
            Ok(Ok(output)) => output,
        };

        // Check output length
        if output.stderr.len() > output_limit_u8 {
            return Err(HandlerError::OutputLimitExceeded);
        }
        if output.stdout.len() > output_limit_u8 {
            return Err(HandlerError::OutputLimitExceeded);
        }

        // Check memory usage
        let memory = memory_controller.memory_stat().max_usage_in_bytes;
        if memory > memory_limit_kib * 1024 {
            return Err(HandlerError::MemoryLimitExceeded);
        }

        let cpu = cpu_controller.cpu().stat;
        let cpu = CpuStats::from_str(&cpu)?;

        // Drop cgroup
        cg.delete()?;

        Ok(super::ExecuteInfo {
            status_code: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
            resource_usage: super::ResourceUsage {
                memory_kib: (memory + 1023) / 1024, // Ceil
                real_time_ms: now.elapsed().as_millis() as u64,
                cpu_time_ms: cpu.usage_usec,
            },
        })
    }

    async fn cleanup(&self, context: &super::ExecutionContext) -> Result<(), super::HandlerError> {
        remove_file(&context.executable_file).await?;
        remove_file(&context.source_file).await?;
        remove_dir(&context.work_dir).await?;

        Ok(())
    }
}
