mod constants;
mod engine;
mod handler;
mod seccomp;
mod utils;

use crate::{engine::Engine, handler::CppHandler};
use shared::{
    protocol::{receive_data, send_data},
    rpc::{JudgeRequest, Language},
};
use tokio_vsock::{VMADDR_CID_HOST, VsockAddr, VsockStream};

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let addr = VsockAddr::new(VMADDR_CID_HOST, constants::DEFAULT_VSOCK_PORT);
    let mut stream = VsockStream::connect(addr).await?;

    loop {
        let data = receive_data(&mut stream).await?;
        let request = postcard::from_bytes::<JudgeRequest>(&data)?;

        // Spawn judging task
        let handle = tokio::spawn(async move {
            match request.language {
                Language::Cpp => {
                    Engine::judge(
                        CppHandler,
                        request,
                        constants::DEFAULT_COMPILE_TIME_LIMIT_MS,
                    )
                    .await
                }
            }
        });

        // Wait judging task
        let response = handle.await?;

        // Send response
        let is_fatal = response.is_fatal_error.unwrap_or(false);
        let result = postcard::to_allocvec(&response)?;
        send_data(&mut stream, &result, result.len() as u32).await?;

        // End process on fatal error
        if is_fatal {
            std::process::exit(1);
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum AgentError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Postcard(#[from] postcard::Error),
    #[error("{0}")]
    Join(#[from] tokio::task::JoinError),
}
