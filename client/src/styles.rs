use iced::{
	border, color, widget::{button, checkbox}, Border, Shadow, Theme
};

pub fn primary_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.palette().primary.into()),
		text_color: color!(0x111111),
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn subdued_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.extended_palette().primary.weak.color.into()),
		text_color: color!(0x111111),
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn success_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.extended_palette().secondary.strong.color.into()),
		text_color: color!(0x111111),
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn green_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.palette().success.into()),
		text_color: color!(0x111111),
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn danger_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.palette().danger.into()),
		text_color: color!(0x111111),
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn invisible_button(theme: &Theme) -> button::Style {
	button::Style {
		background: None,
		text_color: theme.palette().text,
		border: Border::default(),
		shadow: Shadow::default(),
	}
}

pub fn checkbox_style(status: checkbox::Status) -> checkbox::Style {
	checkbox::Style {
		background: match status {
			checkbox::Status::Active { is_checked: true }
			| checkbox::Status::Hovered { is_checked: true } => color!(0x6A4A77).into(),
			_ => color!(0x2C1E31).into(),
		},
		icon_color: color!(0x1B121F),
		border: border::color(color!(0x6A4A77)).width(2).rounded(2),
		text_color: None,
	}
}
