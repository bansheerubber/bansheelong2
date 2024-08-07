use iced::{
	widget::{button, image},
	Element, Padding, Task,
};

use crate::{styles::invisible_button, Message};

pub struct Flavor {
	handle: image::Handle,
	image_index: usize,
	paths: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum FlavorMessage {
	SwitchImage,
}

impl Flavor {
	pub fn new() -> Self {
		let paths = std::fs::read_dir("./flavor-images")
			.unwrap()
			.map(|x| x.unwrap().path().display().to_string())
			.collect::<Vec<_>>();

		let image_index = 18;

		println!("{:?}", paths);

		Self {
			handle: image::Handle::from_path(paths[image_index].clone()),
			image_index,
			paths,
		}
	}

	pub fn update(&mut self, event: FlavorMessage) -> Task<Message> {
		match event {
			FlavorMessage::SwitchImage => {
				self.image_index = (self.image_index + 1) % self.paths.len();
				self.handle = image::Handle::from_path(self.paths[self.image_index].clone());
			}
		}

		Task::none()
	}

	pub fn view(&self) -> Element<FlavorMessage> {
		button(image(self.handle.clone()).width(260).height(215))
			.on_press(FlavorMessage::SwitchImage)
			.padding(Padding::default().top(5).left(5))
			.style(|theme, _status| invisible_button(theme))
			.into()
	}
}
