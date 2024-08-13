use rocket::launch;

mod context;
mod error;
mod routes;
mod services;

pub use context::Context;
pub use error::Error;
pub use error::Result;
use routes::htmx_routes;
use routes::rest_routes;
use routes::ws_routes;

#[launch]
fn rocket() -> _ {
	env_logger::init();
	rocket::build()
		.manage(context::Context::new())
		.mount("/rest/", rest_routes())
		.mount("/ws/", ws_routes())
		.mount("/", htmx_routes())
}
