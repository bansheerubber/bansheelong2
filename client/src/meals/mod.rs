mod component;
mod meal_component;
mod meals_chooser_component;
mod meals_list_component;
mod meals_random_chooser_component;

pub use component::CalendarState;
pub use component::Meals;
pub use component::MealsMessage;
pub use meal_component::meal_contents;
pub use meals_chooser_component::MealsChooser;
pub use meals_list_component::MealsList;
pub use meals_random_chooser_component::RandomMealChooser;

use iced::color;
use iced::Color;
use std::collections::HashMap;
use uuid::Uuid;

pub const COLORS: [Color; 9] = [
	color!(0xE059E0), // #E059E0
	color!(0x9D87FF), // #9D87FF
	color!(0xD796F2), // #D796F2
	color!(0x58B7CE), // #58B7CE
	color!(0x588BCE), // #588BCE
	color!(0x2CBA60), // #2CBA60
	color!(0xDBCD51), // #DBCD51
	color!(0xFF814F), // #FF814F
	color!(0xDD4460), // #DD4460
];

pub fn get_meal_color(meal_id_to_color: &mut HashMap<Uuid, Color>, uuid: &Uuid) -> Color {
	if !meal_id_to_color.contains_key(&uuid) {
		meal_id_to_color.insert(uuid.clone(), COLORS[meal_id_to_color.len() % COLORS.len()]);
	}

	meal_id_to_color.get(uuid).unwrap().clone()
}
