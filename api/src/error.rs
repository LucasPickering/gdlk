use actix_web::{HttpResponse, ResponseError};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "{}", 0)]
    DbError(#[cause] diesel::result::Error),
}

impl From<diesel::result::Error> for ServerError {
    fn from(other: diesel::result::Error) -> Self {
        Self::DbError(other)
    }
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::DbError(diesel::result::Error::NotFound) => {
                HttpResponse::NotFound().into()
            }
            _ => HttpResponse::InternalServerError().into(),
        }
    }
}
