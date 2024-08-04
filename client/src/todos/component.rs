use iced::{
	widget::{container, column, scrollable, text},
	Element, Length, Padding, Theme,
};

use crate::Message;

use super::TodosState;

#[derive(Clone, Debug)]
pub enum TodosMessage {}

pub struct Todos {
	todos_state: TodosState,
	width: u16,
}

impl Todos {
	pub fn new() -> Self {
		Self {
			todos_state: TodosState::default(),
			width: 385,
		}
	}

	fn view_day(&self) -> Element<TodosMessage> {
		container(
			column![
				text("General list:")
			]
		)
			.width(Length::Fill)
			.padding(5)
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
			.into()
	}

	pub fn update(&mut self, _event: TodosMessage) -> Option<Message> {
		None
	}

	pub fn view(&self) -> Element<TodosMessage> {
		scrollable(
			container(self.view_day())
				.width(Length::Fill)
				.padding(Padding::default().top(20).bottom(20).right(15)),
		)
		.width(self.width)
		.into()
	}
}
