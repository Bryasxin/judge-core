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

    let data = receive_data(&mut stream).await?;
    let submission = postcard::from_bytes::<JudgeRequest>(&data)?;

    let result = match submission.language {
        Language::Cpp => {
            Engine::judge(
                CppHandler,
                submission,
                constants::DEFAULT_COMPILE_TIME_LIMIT_MS,
            )
            .await
        }
    };
    let result = postcard::to_allocvec(&result)?;

    send_data(&mut stream, &result, result.len() as u32).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum AgentError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Postcard(#[from] postcard::Error),
}
