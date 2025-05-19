use rocket::{routes, Route};

mod htmx;
mod rest;
mod ws;

use htmx::{
	get_add_recipe, get_login, get_parsed_recipe, get_root, get_style, post_add_ingredient,
	post_add_meal, post_add_step, post_checkbox, post_login,
};
use rest::{get_meals, get_planned_meals, get_shopping_list, post_replace};
use ws::meals_events_stream;

pub fn htmx_routes() -> Vec<Route> {
	routes![
		get_root,
		get_add_recipe,
		get_style,
		post_checkbox,
		get_login,
		post_login,
		post_add_ingredient,
		post_add_step,
		post_add_meal,
		get_parsed_recipe,
	]
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
