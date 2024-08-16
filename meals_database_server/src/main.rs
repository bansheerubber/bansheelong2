use rocket::catch;
use rocket::catchers;
use rocket::launch;

mod auth;
mod context;
mod error;
mod rendering;
mod routes;
mod services;

pub use context::Context;
pub use error::Error;
pub use error::Result;
use rocket::response::Redirect;
use routes::htmx_routes;
use routes::rest_routes;
use routes::ws_routes;

#[catch(401)]
fn not_authorized() -> Redirect {
	Redirect::to("/login")
}

#[launch]
fn rocket() -> _ {
	env_logger::init();
	rocket::build()
		.manage(context::Context::new())
		.mount("/rest/", rest_routes())
		.mount("/ws/", ws_routes())
		.mount("/", htmx_routes())
		.register("/", catchers![not_authorized])
}
