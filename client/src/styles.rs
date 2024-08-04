use iced::{color, widget::button, Border, Shadow, Theme};

pub fn primary_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.palette().primary.into()),
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
