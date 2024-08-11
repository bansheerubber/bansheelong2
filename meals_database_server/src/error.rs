use rocket::serde::json::json;
use rocket::{
	http::{ContentType, Status},
	response::{self, Responder, Response},
	Request,
};
use std::io::Cursor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("authentication error")]
	AuthenticationError { message: String },

	#[error("authentication error")]
	AuthorizationError { message: String },

	#[error("internal server error")]
	InternalServerError { message: String },

	#[error("payload problem")]
	PayloadProblem { message: String },
}

impl Error {
	pub fn get_message<'a>(&'a self) -> Option<&'a str> {
		match self {
			Error::AuthenticationError { message } => Some(message),
			Error::AuthorizationError { message } => Some(message),
			Error::InternalServerError { message } => Some(message),
			Error::PayloadProblem { message } => Some(message),
		}
	}

	pub fn get_status_code(&self) -> Status {
		match self {
			Error::AuthenticationError { .. } => Status::Unauthorized,
			Error::AuthorizationError { .. } => Status::Forbidden,
			Error::InternalServerError { .. } => Status::InternalServerError,
			Error::PayloadProblem { .. } => Status::BadRequest,
		}
	}
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
	fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
		match self.get_message() {
			Some(message) => {
				let body = json!({ "error": { "message": message, } }).to_string();

				if self.get_status_code() == Status::InternalServerError {
					log::error!("{:?}", self);
				}

				Ok(Response::build()
					.status(self.get_status_code())
					.header(ContentType::JSON)
					.sized_body(body.len(), Cursor::new(body))
					.finalize())
			}
			None => {
				log::error!("{:?}", self);
				Status::InternalServerError.respond_to(req)
			}
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
