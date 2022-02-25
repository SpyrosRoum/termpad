use axum::response::IntoResponse;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Paste not found")]
    NotFound,
    #[error("{0}")]
    Http(#[from] axum::http::Error),
    #[error("{0}")]
    Other(&'static str),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        // TODO Make `NotFound` redirect to `/usage?not_found=true`
        todo!()
    }
}
