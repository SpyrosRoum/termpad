use axum::{
    http::{StatusCode, Uri},
    response::{IntoResponse, Redirect},
};

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
        let (status, err_message) = match self {
            Error::NotFound => {
                let uri = Uri::from_static("/usage?not_found=true");
                return Redirect::to(uri).into_response();
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"),
        };

        (status, err_message).into_response()
    }
}
