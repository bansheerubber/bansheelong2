#![feature(let_chains)]

use anyhow::{bail, Context, Result};
use futures::future::join;
use std::{
	fs::read_dir,
	os::unix::process::ExitStatusExt,
	path::Path,
	process::{exit, Stdio},
	sync::Arc,
	time::Duration,
};
use storage_server::{
	HardDriveStatus, HardDriveStatusName, JobStatusFlags, ZPoolStatus, ZPoolStatusName,
};
use thiserror::Error;
use tokio::{
	net::{tcp::OwnedWriteHalf, TcpListener},
	process::Command,
	sync::RwLock,
	time::sleep,
};

#[derive(Debug, Error)]
enum Error {
	#[error("child returned non-zero exit code: {0}")]
	NonZeroExitCode(i32),
}

async fn run_command(command: &mut Command) -> Result<String> {
	let child = command
		.stdout(Stdio::piped())
		.spawn()?
		.wait_with_output()
		.await?;

	if !child.status.success() {
		bail!(Error::NonZeroExitCode(child.status.into_raw()));
	}

	Ok(String::from_utf8(child.stdout)?)
}

fn get_hard_drive_status(line: &str) -> Result<Option<HardDriveStatus>> {
	if line.contains("state:") {
		return Ok(None);
	}

	let hard_drive_status = ["ONLINE", "FAULTED", "DEGRADED", "UNAVAIL", "OFFLINE"]
		.iter()
		.find(|&&item| line.contains(item));

	let hard_drive_status: HardDriveStatusName = if let Some(hard_drive_status) = hard_drive_status
	{
		(*hard_drive_status).try_into()?
	} else {
		return Ok(None);
	};

	let split = line
		.split(" ")
		.map(|item| item.replace("\t", "").trim().to_string())
		.filter(|item| item.len() != 0)
		.collect::<Vec<_>>();

	Ok(Some(HardDriveStatus {
		status: hard_drive_status,
		hard_drive_name: split
			.get(0)
			.context("Could not get hard drive name")?
			.to_string(),
		checksum_errors: split[2].parse()?,
		read_errors: split[3].parse()?,
		write_errors: split[4].parse()?,
	}))
}

async fn get_zpool_status() -> Result<ZPoolStatus> {
	let stdout = run_command(Command::new("zpool").arg("status")).await?;

	let mut hard_drive_statuses = vec![];
	let mut status = None;

	for line in stdout.split('\n') {
		if line.contains("scan:") && line.contains("scrub in progress") {
			status = Some(ZPoolStatusName::Scrubbing);
		}

		let hard_drive_status = get_hard_drive_status(line)?;
		if let Some(hard_drive_status) = hard_drive_status {
			if hard_drive_status.is_error() {
				status = Some(ZPoolStatusName::HardDriveError);
			}

			hard_drive_statuses.push(hard_drive_status);
		}

		if line.contains("errors:") && line != "errors: No known data errors" {
			status = Some(ZPoolStatusName::Error);
		}
	}

	Ok(ZPoolStatus {
		hard_drive_statuses,
		status: status.unwrap_or(ZPoolStatusName::Safe),
	})
}

async fn get_disk_usage() -> Result<(u64, u64)> {
	let stdout = run_command(Command::new("df").arg("-B1")).await?;

	for line in stdout.split("\n") {
		if !line.contains("bansheerubber") {
			continue;
		}

		let items = line
			.split(" ")
			.filter(|item| item.len() != 0)
			.map(|item| item.to_string())
			.collect::<Vec<_>>();

		let total_size = items.get(1).context("Could not get used size")?.parse()?;
		let used_size = items.get(2).context("Could not get used size")?.parse()?;
		return Ok((used_size, total_size));
	}

	bail!("Could not read disk usage")
}

async fn get_btrfs_disk_usage() -> Result<(u64, u64)> {
	let stdout = run_command(
		Command::new("btrfs")
			.arg("fi")
			.arg("usage")
			.arg("-b")
			.arg("/bansheebtrfs/"),
	)
	.await?;

	let mut total_size = 0;
	let mut used_size = 0;
	for line in stdout.split("\n") {
		let split = line
			.split(" ")
			.map(|item| item.to_string())
			.filter(|item| item.trim().len() > 0)
			.collect::<Vec<_>>();

		if line.contains("Device size:") {
			total_size = split
				.last()
				.context("Could not get last in spliit")?
				.parse()?;
		} else if line.contains("Used:\t") {
			used_size = split
				.last()
				.context("Could not get last in spliit")?
				.parse()?;
		}
	}

	if total_size == 0 && used_size == 0 {
		bail!("Could not read btrfs read disk usage")
	} else {
		return Ok((used_size, total_size));
	}
}

async fn get_job_flags() -> JobStatusFlags {
	let mut result = JobStatusFlags::IDLE;

	match get_zpool_status().await {
		Ok(status) => match status.status {
			ZPoolStatusName::Error => result |= JobStatusFlags::ZPOOL_ERROR,
			ZPoolStatusName::HardDriveError => result |= JobStatusFlags::ZPOOL_HARD_DRIVE_RW_ERROR,
			ZPoolStatusName::Scrubbing => result |= JobStatusFlags::ZPOOL_SCRUBBING,
			_ => {}
		},
		Err(_) => result |= JobStatusFlags::ZPOOL_HARD_DRIVE_STATE_ERROR,
	}

	if Path::new("/home/me/bansheestorage/writing-git-backup").exists() {
		result |= JobStatusFlags::SYNCING_GITHUB;
	}

	if Path::new("/home/me/bansheestorage/writing-btrbk").exists() {
		result |= JobStatusFlags::WRITING_BTRBK;
	}

	result
}

fn get_btrfs_backup_count() -> Result<u64> {
	let mut total = 0;
	for path in read_dir("/bansheebtrfs/")? {
		if path?
			.path()
			.to_str()
			.context("Could not convert to string")?
			.contains("home_backup")
		{
			total += 1;
		}
	}

	Ok(total)
}

async fn write_socket(socket: &OwnedWriteHalf, message: &str) -> Result<(), tokio::io::Error> {
	loop {
		// keep looping until we can write
		socket.writable().await?;

		match socket.try_write(message.as_bytes()) {
			Ok(_) => {
				break;
			}
			Err(ref error) if error.kind() == tokio::io::ErrorKind::WouldBlock => {
				continue;
			}
			Err(error) => return Err(error),
		}
	}

	Ok(())
}

#[tokio::main]
async fn main() {
	let zpool_status = get_zpool_status().await;
	let disk_usage = get_disk_usage().await;
	let btrfs_disk_usage = get_btrfs_disk_usage().await;

	let sockets = Arc::new(RwLock::new(vec![]));
	let message = Arc::new(RwLock::new(String::new()));

	join(
		async {
			let sockets = sockets.clone();

			let Ok(listener) = TcpListener::bind("0.0.0.0:3003").await else {
				println!("Could not open socket");
				exit(1);
			};

			loop {
				let Ok(socket) = listener.accept().await else {
					println!("Could not accept socket");
					continue;
				};

				let (read, write) = socket.0.into_split();

				let message = message.write().await;
				if let Err(error) = write_socket(&write, &message).await {
					println!("Socket write error {:?}", error);
				}

				let mut sockets = sockets.write().await;
				sockets.push((read, write));
			}
		},
		async {
			let sockets = sockets.clone();
			loop {
				sleep(Duration::from_secs(5)).await;

				let job_status = get_job_flags().await;

				let (used_size, total_size) = match get_disk_usage().await {
					Ok(sizes) => sizes,
					Err(error) => {
						println!("get_disk_usage error {:?}", error);
						continue;
					}
				};

				let (btrfs_used_size, btrfs_total_size) = match get_btrfs_disk_usage().await {
					Ok(sizes) => sizes,
					Err(error) => {
						println!("get_btrfs_disk_usage error {:?}", error);
						continue;
					}
				};

				let btrfs_backup_count = match get_btrfs_backup_count() {
					Ok(sizes) => sizes,
					Err(error) => {
						println!("get_btrfs_backup_count error {:?}", error);
						continue;
					}
				};

				let mut message = message.write().await;
				*message = format!(
					"{} {} {} {} {} {}\n",
					job_status.bits(),
					used_size,
					total_size,
					btrfs_used_size,
					btrfs_total_size,
					btrfs_backup_count,
				);

				let mut dead_sockets = vec![];
				let mut sockets = sockets.write().await;
				for i in 0..sockets.len() {
					let (_, write) = &sockets[i];
					if let Err(error) = write_socket(&write, &message).await {
						println!("socket write error {:?}", error);
						dead_sockets.push(i);
					}
				}

				for dead_socket in dead_sockets {
					sockets.remove(dead_socket);
				}
			}
		},
	)
	.await;

	println!("{:?} {:?} {:?}", zpool_status, disk_usage, btrfs_disk_usage);
}
