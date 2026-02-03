mod engine;
mod handler;
mod seccomp;
mod utils;

use crate::{engine::Engine, handler::CppHandler};
use shared::{
    protocol::{receive_data, send_data},
    rpc::{Language, Submission},
};
use tokio_vsock::{VMADDR_CID_HOST, VsockAddr, VsockStream};

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let addr = VsockAddr::new(VMADDR_CID_HOST, 9999);
    let mut stream = VsockStream::connect(addr).await?;

    let data = receive_data(&mut stream).await?;
    let submission = postcard::from_bytes::<Submission>(&data)?;

    let result = match submission.language {
        Language::Cpp => Engine::judge(CppHandler, submission, 60000).await,
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
