use std::str::FromStr;

use rocket::{
	request::{self, FromRequest, Outcome},
	Request,
};
use uuid::Uuid;

use crate::{Context, Error};

#[derive(Debug)]
pub struct User {
	name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
	type Error = Error;

	async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Error> {
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
