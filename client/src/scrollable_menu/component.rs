use std::{
	cell::RefCell,
	time::{Duration, Instant},
};

use iced::{
	border::Radius,
	color,
	task::Handle,
	widget::{column, scrollable, Button, Space},
	Border, Color, Element, Length, Padding, Task, Theme,
};

use crate::{meals::MealsMessage, Message};

pub struct ScrollableMenu {
	current_y: f32,
	pub id: scrollable::Id,
	last_interaction: Instant,
	reset_handle: Option<Handle>,

	// TODO this sucks
	number_of_buttons: RefCell<usize>,
}

#[derive(Clone, Debug)]
pub enum ScrollableMenuMessage {
	OnScroll {
		id: scrollable::Id,
		viewport: scrollable::Viewport,
	},
	Reset {
		id: scrollable::Id,
	},
}

impl ScrollableMenuMessage {
	pub fn get_id(&self) -> &scrollable::Id {
		match self {
			ScrollableMenuMessage::OnScroll { id, .. } => &id,
			ScrollableMenuMessage::Reset { id } => &id,
		}
	}
}

fn scrollbar_visible(theme: &Theme) -> scrollable::Style {
	let rail = scrollable::Rail {
		background: None,
		border: Border::default(),
		scroller: scrollable::Scroller {
			color: color!(0x261A2B),
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(5),
			},
		},
	};

	return scrollable::Style {
		container: theme.palette().background.into(),
		vertical_rail: rail.clone(),
		horizontal_rail: rail,
		gap: None,
	};
}

fn scrollbar_invisible(theme: &Theme) -> scrollable::Style {
	let rail = scrollable::Rail {
		background: None,
		border: Border::default(),
		scroller: scrollable::Scroller {
			color: Color::TRANSPARENT,
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(5),
			},
		},
	};

	return scrollable::Style {
		container: theme.palette().background.into(),
		vertical_rail: rail.clone(),
		horizontal_rail: rail,
		gap: None,
	};
}

impl ScrollableMenu {
	pub fn new() -> (Self, Task<Message>) {
		let id = scrollable::Id::unique();
		(
			Self {
				current_y: 0.0,
				id: id.clone(),
				last_interaction: Instant::now() - Duration::from_secs(50),
				reset_handle: None,

				number_of_buttons: RefCell::new(0),
			},
			Task::future(async {
				tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
				Message::Meals(MealsMessage::Scrollable(ScrollableMenuMessage::Reset {
					id,
				}))
			}),
		)
	}

	fn buttons_height(&self) -> f32 {
		let number = *self.number_of_buttons.borrow() as f32;
		if number > 0.0 {
			35.0 * number + 5.0 * (number - 1.0) + 20.0
		} else {
			0.0
		}
	}

	pub fn update(&mut self, message: ScrollableMenuMessage) -> Task<Message> {
		match message {
			ScrollableMenuMessage::OnScroll { viewport, .. } => {
				if let Some(reset_handle) = &self.reset_handle
					&& !reset_handle.is_aborted()
				{
					reset_handle.abort();
				}

				let id = self.id.clone();
				let (task, handle) = Task::future(async {
					tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

					Message::Meals(MealsMessage::Scrollable(ScrollableMenuMessage::Reset {
						id,
					}))
				})
				.abortable();

				self.reset_handle = Some(handle);
				self.current_y = viewport.absolute_offset().y;
				self.last_interaction = Instant::now();

				task
			}
			ScrollableMenuMessage::Reset { .. } => {
				let height = self.buttons_height();
				if self.current_y < height {
					scrollable::scroll_to(
						self.id.clone(),
						scrollable::AbsoluteOffset { x: 0.0, y: height },
					)
				} else {
					Task::none()
				}
			}
		}
	}

	pub fn view<'a, Message: 'a>(
		&'a self,
		contents: Element<'a, Message>,
		buttons: Vec<Button<'a, Message>>,
		min_height: u16,
	) -> Element<'a, Message>
	where
		Message: Clone + From<ScrollableMenuMessage>,
	{
		let size = buttons.len();
		*self.number_of_buttons.borrow_mut() = size;

		let buttons = buttons
			.into_iter()
			.map(|button: Button<'a, Message>| button.height(35).into());

		let mut column = column(vec![column(buttons)
			.spacing(5)
			.padding(Padding::default().bottom(if size != 0 { 20 } else { 0 }))
			.into()])
		.padding(Padding::default().top(20).bottom(20).right(15));

		column = column.push(contents);
		column = column.push(Space::new(0, min_height.saturating_sub(20)));

		scrollable(column)
			.id(self.id.clone())
			.on_scroll(|viewport| {
				ScrollableMenuMessage::OnScroll {
					id: self.id.clone(),
					viewport,
				}
				.into()
			})
			.width(Length::Fill)
			.height(Length::Fill)
			.style(|theme, _status| {
				if Instant::now() - self.last_interaction < Duration::from_secs(5) {
					scrollbar_visible(theme)
				} else {
					scrollbar_invisible(theme)
				}
			})
			.into()
	}
}
