use std::ops::Add;

use iced::{
	widget::{column, container, row, text, Space},
	Element, Length, Padding, Task, Theme,
};
use storage_server::JobStatusFlags;

use crate::Message;

#[derive(Clone, Debug)]
pub enum StorageMessage {
	Update { data: StorageData },
}

#[derive(Clone, Debug, Default)]
pub struct StorageData {
	pub btrfs_backup_count: u64,
	pub btrfs_total_size: u64,
	pub btrfs_used_size: u64,
	pub job_flags: JobStatusFlags,
	pub total_size: u64,
	pub used_size: u64,
}

pub struct Storage {
	data: Option<StorageData>,
	ellipses: usize,
}

fn format_size(size: u64) -> String {
	if size == 0 {
		return "0T".into();
	}

	let gigabytes = size / 1_000_000_000;
	if gigabytes < 1_500 {
		format!("{:.1}T", gigabytes as f64 / 1_000.0)
	} else {
		format!("{}T", gigabytes / 1_000)
	}
}

impl Storage {
	pub fn new() -> Storage {
		Storage {
			data: None,
			ellipses: 0,
		}
	}

	fn status_text(&self) -> String {
		let ellipses = ".".repeat(self.ellipses.add(1).min(4));

		let Some(data) = self.data.as_ref() else {
			return "Not connected".into();
		};

		if data.job_flags.contains(JobStatusFlags::GENERAL_ERROR) {
			String::from("Error")
		} else if data.job_flags.contains(JobStatusFlags::SYNCING_GITHUB) {
			String::from("Syncing GitHub to backup") + &ellipses
		} else if data.job_flags.contains(JobStatusFlags::ZPOOL_ERROR) {
			String::from("ZPool error")
		} else if data
			.job_flags
			.contains(JobStatusFlags::ZPOOL_HARD_DRIVE_RW_ERROR)
		{
			String::from("Hard drive r/w/c error")
		} else if data
			.job_flags
			.contains(JobStatusFlags::ZPOOL_HARD_DRIVE_STATE_ERROR)
		{
			String::from("Hard drive error")
		} else if data.job_flags.contains(JobStatusFlags::ZPOOL_SCRUBBING) {
			String::from("Scrubbing") + &ellipses
		} else if data.job_flags.contains(JobStatusFlags::WRITING_BTRBK) {
			String::from("Writing btrfs backup") + &ellipses
		} else {
			String::from("Idle")
		}
	}

	pub fn update(&mut self, event: StorageMessage) -> Task<Message> {
		match event {
			StorageMessage::Update { data } => self.data = Some(data),
		}

		Task::none()
	}

	pub fn view(&self) -> Element<StorageMessage> {
		container(
			container(
				column![
					row![
						text!(
							"{} backups",
							self.data
								.as_ref()
								.unwrap_or(&StorageData::default())
								.btrfs_backup_count
						),
						Space::with_width(18),
						text!(
							"{}/{}",
							format_size(
								self.data
									.as_ref()
									.unwrap_or(&StorageData::default())
									.btrfs_used_size
							),
							format_size(
								self.data
									.as_ref()
									.unwrap_or(&StorageData::default())
									.btrfs_total_size
							),
						),
						Space::with_width(18),
						text!(
							"{}/{}",
							format_size(
								self.data
									.as_ref()
									.unwrap_or(&StorageData::default())
									.used_size
							),
							format_size(
								self.data
									.as_ref()
									.unwrap_or(&StorageData::default())
									.total_size
							),
						),
					]
					.width(Length::Fill),
					text(self.status_text()),
				]
				.width(240),
			)
			.padding(10)
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into()),
		)
		.padding(Padding::default().top(20).left(5))
		.into()
	}
}
