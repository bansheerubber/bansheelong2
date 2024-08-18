use meals_database::MealPlan;
use rocket::{
	get, post,
	serde::json::{json, Json, Value},
	State,
};
use serde::Serialize;

use crate::{auth::RestUser, Context, Result};

#[derive(Debug, Serialize)]
pub struct Response {}

#[post("/meals/replace", data = "<meal_plan>")]
pub async fn post_replace(
	context: &State<Context>,
	meal_plan: Json<MealPlan>,
	_user: RestUser,
) -> Result<Value> {
	context
		.meals_database
		.write()
		.await
		.replace(meal_plan.into_inner());

	Ok(json!(Response {}))
}

#[get("/meals/all")]
pub async fn get_meals(context: &State<Context>, _user: RestUser) -> Result<Value> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();
	Ok(json!(&*meal_plan))
}

#[get("/meals/shopping-list")]
pub async fn get_shopping_list(context: &State<Context>, _user: RestUser) -> Result<Value> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();
	Ok(json!(meal_plan.shopping_list))
}

#[get("/meals/planned-meals")]
pub async fn get_planned_meals(context: &State<Context>, _user: RestUser) -> Result<Value> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();
	Ok(json!(meal_plan.planned_meals))
}
