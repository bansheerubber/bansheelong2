use rocket::{routes, Route};

mod htmx;
mod rest;
mod ws;

use htmx::{get_root, get_style, post_checkbox};
use rest::{get_meals, get_planned_meals, get_shopping_list, post_replace};
use ws::meals_events_stream;

pub fn htmx_routes() -> Vec<Route> {
	routes![get_root, get_style, post_checkbox]
}

pub fn rest_routes() -> Vec<Route> {
	routes![
		get_meals,
		get_planned_meals,
		get_shopping_list,
		post_replace,
	]
}

pub fn ws_routes() -> Vec<Route> {
	routes![meals_events_stream]
}
