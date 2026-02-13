//! Firecracker API client
use http_body_util::{BodyExt, Full};
use hyper::{
    Method, Request, Response, StatusCode, Uri,
    body::{Bytes, Incoming},
};
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector, Uri as UnixUri};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FirecrackerApiClient {
    client: Client<UnixConnector, Full<Bytes>>,
    socket_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Hyper http error: {0}")]
    HyperHttp(#[from] hyper::http::Error),

    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("Request error: {0}")]
    Request(#[from] hyper_util::client::legacy::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Firecracker API error: {0}")]
    Firecracker(String),
}

impl FirecrackerApiClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            client: Client::unix(),
            socket_path: socket_path.into(),
        }
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Full<Bytes>,
    ) -> Result<Response<Incoming>, ApiError> {
        let url: Uri = UnixUri::new(&self.socket_path, path).into();

        let req = Request::builder()
            .method(method)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(body)?;

        let response = self.client.request(req).await?;
        Ok(response)
    }

    async fn parse_response<T>(
        &self,
        response: Response<Incoming>,
        expected_status: StatusCode,
    ) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();
        let body = response.into_body().collect().await?.to_bytes();

        match status {
            s if s == expected_status => {
                let result: T = serde_json::from_slice(&body)?;
                Ok(result)
            }
            StatusCode::BAD_REQUEST => {
                let error: crate::dto::Error = serde_json::from_slice(&body)?;
                Err(ApiError::InvalidInput(error.fault_message))
            }
            _ => {
                let error: crate::dto::Error = serde_json::from_slice(&body)?;
                Err(ApiError::Firecracker(error.fault_message))
            }
        }
    }

    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    async fn get<T>(&self, path: &str, expected_status: StatusCode) -> Result<T, ApiError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result = self
            .request(Method::GET, path, Full::new(Bytes::new()))
            .await?;
        self.parse_response(result, expected_status).await
    }

    async fn put<T>(
        &self,
        path: &str,
        req: Full<Bytes>,
        expected_status: StatusCode,
    ) -> Result<T, ApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let result = self.request(Method::GET, path, req).await?;
        self.parse_response(result, expected_status).await
    }

    async fn patch<T>(
        &self,
        path: &str,
        req: Full<Bytes>,
        expected_status: StatusCode,
    ) -> Result<T, ApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let result = self.request(Method::PATCH, path, req).await?;
        self.parse_response(result, expected_status).await
    }
}
