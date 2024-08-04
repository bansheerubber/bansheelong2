#![feature(let_chains)]

use anyhow::Result;
use iced::{
	font::{Family, Stretch, Style, Weight},
	Font,
};

mod calendar;
mod meals;
mod scrollable_menu;
mod styles;
mod todos;
mod util;
mod weather;
mod widgets;
mod window;
mod storage;

pub use window::Message;

pub const WINDOW_HEIGHT: f32 = 320.0;

pub const NOTOSANS_BOLD: Font = Font {
	family: Family::Name("Noto Sans"),
	weight: Weight::Bold,
	stretch: Stretch::Normal,
	style: Style::Normal,
};

pub const ICONS: Font = Font {
	family: Family::Name("Material Icons"),
	weight: Weight::Normal,
	stretch: Stretch::Normal,
	style: Style::Normal,
};

pub fn pt(number: u32) -> f32 {
	(number as f32) * (42.2 / 55.0)
}

fn main() -> Result<()> {
	env_logger::init();

	iced::application("Test", window::Window::update, window::Window::view)
		.font(include_bytes!("../../fonts/NotoSans-Medium.ttf"))
		.font(include_bytes!("../../fonts/NotoSans-Bold.ttf"))
		.font(include_bytes!("../../fonts/NotoSans-Regular.ttf"))
		.font(include_bytes!("../../fonts/MaterialIcons-Regular.ttf"))
		.default_font(Font {
			family: Family::Name("Noto Sans"),
			weight: Weight::Medium,
			stretch: Stretch::Normal,
			style: Style::Normal,
		})
		.theme(window::Window::theme)
		.subscription(window::Window::subscription)
		.antialiasing(false)
		.resizable(false)
		.decorations(false)
		.window_size((1480.0, 320.0))
		.run_with(window::Window::new)?;

	Ok(())
}
