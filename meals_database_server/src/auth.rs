use rocket::{
	request::{self, FromRequest, Outcome},
	Request,
};
use std::str::FromStr;
use uuid::Uuid;

use crate::{Context, Error};

#[derive(Debug)]
pub struct User {
	#[allow(dead_code)]
	name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
	type Error = Error;

	async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Error> {
		if std::env::var("BANSHEELONG2_DEBUG").is_ok() {
			return Outcome::Success(User { name: "me".into() })
		}

		let Some(sid_cookie) = request.cookies().get("SID") else {
			let error = Error::AuthenticationError {
				message: "No cookie found".into(),
			};
			return Outcome::Error((error.get_status_code(), error));
		};

		let context = match request.rocket().state::<Context>() {
			Some(context) => context,
			None => {
				let error = Error::InternalServerError {
					message: "Could not get context".into(),
				};

				return Outcome::Error((error.get_status_code(), error));
			}
		};

		let valid_sids = context.valid_sids.read().await;

		if valid_sids.contains(&Uuid::from_str(sid_cookie.value()).unwrap()) {
			Outcome::Success(User { name: "me".into() })
		} else {
			let error = Error::AuthenticationError {
				message: "Invalid cookie".into(),
			};
			return Outcome::Error((error.get_status_code(), error));
		}
	}
}

pub struct RestUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RestUser {
	type Error = Error;

	async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Error> {
		let Some(token) = request.headers().get_one("authorization") else {
			let error = Error::AuthenticationError {
				message: "Could not find authorization header".into(),
			};
			return Outcome::Error((error.get_status_code(), error));
		};

		let split = token.split(" ").collect::<Vec<_>>();
		if split.len() != 2 {
			let error = Error::AuthenticationError {
				message: "Could not find authorization token".into(),
			};
			return Outcome::Error((error.get_status_code(), error));
		}

		let expected_token = std::fs::read_to_string("./auth-token").unwrap();
		if split[1] == expected_token.trim() {
			Outcome::Success(RestUser)
		} else {
			let error = Error::AuthenticationError {
				message: "Invalid auth token".into(),
			};
			return Outcome::Error((error.get_status_code(), error));
		}
	}
}
