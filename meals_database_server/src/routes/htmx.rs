use maud::{html, Markup};
use rocket::{
	form::Form,
	get,
	http::{Cookie, CookieJar},
	post,
	response::{content::RawCss, Redirect},
	FromForm, State,
};
use uuid::Uuid;

use crate::{
	auth::User,
	rendering::{render_checkbox, render_meal, render_shopping_list, root},
	Context, Result,
};

#[get("/")]
pub async fn get_root(context: &State<Context>, _user: User) -> Result<Markup> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();

	let shopping_lists = &meal_plan.shopping_list;
	let planned_meals = &meal_plan.planned_meals;
	let all_meals = &meal_plan.all_meals;

	let shopping_list_markup = if shopping_lists.len() == 0 {
		html! {
			div class="flex flex-col items-center gap-4 pt-6 px-4" {
				div class="shopping-list flex gap-2 text-lg w-full sm:w-[500px] p-2 justify-center" {
					"No shopping lists available"
				}
			}
		}
	} else {
		html! {
			@for (shopping_list_index, shopping_list) in shopping_lists.iter().enumerate() {
				(render_shopping_list(shopping_list_index, &shopping_list))
			}
		}
	};

	let mut meals = vec![];
	for meals_for_day in planned_meals.values() {
		for meal_stub in meals_for_day.iter() {
			if meal_stub.leftovers {
				continue;
			}

			meals.push((meal_stub.date, all_meals.get(&meal_stub.id).unwrap()));
		}
	}

	meals.sort_by(|(date1, _), (date2, _)| date1.cmp(date2));

	Ok(root(html! {
		div class="flex flex-col items-center gap-4 pt-6 px-4" {
			(shopping_list_markup)

			@for meal in meals.iter() {
				(render_meal(meal.1))
			}
		}
	}))
}

#[get("/style.css")]
pub fn get_style() -> RawCss<String> {
	RawCss(std::fs::read_to_string("./meals_database_server/output.css").unwrap())
}

#[derive(Debug, FromForm)]
pub struct CheckboxState {
	checked: bool,
	name: String,
	shopping_list_index: usize,
}

#[post("/update-checkbox", data = "<checkbox>")]
pub async fn post_checkbox(
	context: &State<Context>,
	checkbox: Form<CheckboxState>,
	_user: User,
) -> Result<Markup> {
	let database = context.meals_database.write().await;
	let mut meal_plan = database.get_mut();

	let shopping_list = meal_plan
		.shopping_list
		.get_mut(checkbox.shopping_list_index)
		.unwrap();

	let shopping_list_item = shopping_list
		.items
		.iter_mut()
		.find(|item| item.name == checkbox.name)
		.unwrap();

	shopping_list_item.have = checkbox.checked;

	let shopping_list_item = shopping_list_item.clone();

	drop(meal_plan);

	database.save();

	Ok(html! { (render_checkbox(checkbox.shopping_list_index, &shopping_list_item)) })
}

#[get("/login")]
pub fn get_login() -> Markup {
	root(html! {
		div class="flex justify-center w-full mt-4 p-6" {
			form
				class="flex flex-col gap-2 w-full sm:w-[500px]"
				action="/login"
				method="post"
			{
				input name="username" type="text" placeholder="username";
				input name="password" type="password" placeholder="password";
				button class="submit-button" type="submit" { "Submit" }
			}
		}
	})
}

#[derive(Debug, FromForm)]
pub struct LoginForm {
	username: String,
	password: String,
}

#[post("/login", data = "<login>")]
pub async fn post_login(
	context: &State<Context>,
	login: Form<LoginForm>,
	cookies: &CookieJar<'_>,
) -> Redirect {
	let password = std::fs::read_to_string("password").unwrap();
	if login.username != "me" && login.password != password {
		return Redirect::to("/login");
	}

	let sid = Uuid::new_v4();
	cookies.add(Cookie::build(("SID", sid.to_string())).expires(None));

	let mut valid_sids = context.valid_sids.write().await;
	valid_sids.insert(sid.clone());

	Redirect::to("/")
}
