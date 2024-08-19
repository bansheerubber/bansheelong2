use anyhow::bail;
use bitflags::bitflags;

bitflags! {
	#[derive(Clone, Debug, Default)]
	pub struct JobStatusFlags: u64 {
		const IDLE                           = 0;
		const GENERAL_ERROR	                 = 1 << 0;
		const SYNCING_GITHUB                 = 1 << 1;
		const ZPOOL_ERROR                    = 1 << 2;
		const ZPOOL_HARD_DRIVE_RW_ERROR      = 1 << 3;
		const ZPOOL_HARD_DRIVE_STATE_ERROR   = 1 << 4;
		const ZPOOL_SCRUBBING                = 1 << 5;
		const WRITING_BTRBK                  = 1 << 6;
	}
}

#[derive(Clone, Debug, Default)]
pub struct StorageData {
	pub btrfs_backup_count: u64,
	pub btrfs_total_size: u64,
	pub btrfs_used_size: u64,
	pub dailies: u8,
	pub job_flags: JobStatusFlags,
	pub total_size: u64,
	pub used_size: u64,
	pub weeklies: u8,
}

#[derive(Debug)]
pub enum ZPoolStatusName {
	Error,
	HardDriveError,
	Scrubbing,
	Safe,
}

#[derive(Debug)]
pub struct ZPoolStatus {
	pub hard_drive_statuses: Vec<HardDriveStatus>,
	pub status: ZPoolStatusName,
}

#[derive(Debug)]
pub enum HardDriveStatusName {
	Degraded,
	Faulted,
	Offline,
	Online,
	Unavailable,
}

#[derive(Debug)]
pub struct HardDriveStatus {
	pub hard_drive_name: String,
	pub status: HardDriveStatusName,
	pub checksum_errors: usize,
	pub read_errors: usize,
	pub write_errors: usize,
}

impl TryFrom<&str> for HardDriveStatusName {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"ONLINE" => Ok(HardDriveStatusName::Online),
			"FAULTED" => Ok(HardDriveStatusName::Faulted),
			"DEGRADED" => Ok(HardDriveStatusName::Degraded),
			"UNAVAIL" => Ok(HardDriveStatusName::Unavailable),
			"OFFLINE" => Ok(HardDriveStatusName::Offline),
			_ => bail!("Could not decode zpool hard drive status"),
		}
	}
}

impl HardDriveStatus {
	pub fn is_error(&self) -> bool {
		if self.checksum_errors != 0 || self.read_errors != 0 || self.write_errors != 0 {
			true
		} else {
			false
		}
	}
}
