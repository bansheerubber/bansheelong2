use iced::{
	advanced::{
		layout::{Limits, Node},
		renderer::{self, Quad, Style},
		widget::Tree,
		Layout, Widget,
	},
	border,
	mouse::Cursor,
	Color, Element, Length, Size,
};

pub struct Circle {
	color: Color,
	radius: f32,
}

pub fn circle(color: Color, radius: f32) -> Circle {
	Circle { color, radius }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Circle
where
	Renderer: renderer::Renderer,
{
	fn size(&self) -> Size<Length> {
		Size {
			width: Length::Shrink,
			height: Length::Shrink,
		}
	}

	fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, _limits: &Limits) -> Node {
		Node::new(Size::new(self.radius * 2.0, self.radius * 2.0))
	}

	fn draw(
		&self,
		_tree: &Tree,
		renderer: &mut Renderer,
		_theme: &Theme,
		_style: &Style,
		layout: Layout<'_>,
		_cursor: Cursor,
		_viewport: &iced::Rectangle,
	) {
		renderer.fill_quad(
			Quad {
				bounds: layout.bounds(),
				border: border::rounded(self.radius),
				..Quad::default()
			},
			self.color,
		);
	}
}

impl<'a, Message, Theme, Renderer> From<Circle> for Element<'a, Message, Theme, Renderer>
where
	Renderer: renderer::Renderer,
{
	fn from(circle: Circle) -> Self {
		Self::new(circle)
	}
}
