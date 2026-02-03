use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_vsock::VsockStream;

/// Send data to vsock stream
///
/// Warning: Using private protocol, so do not send data without using this function.
pub async fn send_data(
    stream: &mut VsockStream,
    data: &[u8],
    len: u32,
) -> Result<(), std::io::Error> {
    stream.write_u32_le(len).await?;
    stream.write_all(data).await?;

    Ok(())
}

/// Receive data from vsock stream
///
/// Warning: Using private protocol, so do not receive data without using this function.
pub async fn receive_data(stream: &mut VsockStream) -> Result<Vec<u8>, std::io::Error> {
    let len = stream.read_u32_le().await?;
    let mut buf = vec![0; len as usize];
    stream.read_exact(&mut buf).await?;

    Ok(buf)
}
