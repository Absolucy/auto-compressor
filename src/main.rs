// SPDX-License-Identifier: Zlib
cfg_if::cfg_if! {
	if #[cfg(windows)] {
		mod windows;
		pub use crate::windows::*;
	}
}

mod compress;
mod config;

use self::config::Config;
use color_eyre::eyre::{ContextCompat, Result, WrapErr};
use log::{debug, error, info, LevelFilter};
use std::{path::Path, time::Duration};

fn main() -> Result<()> {
	color_eyre::install()?;
	pretty_env_logger::formatted_builder()
		.filter_level(LevelFilter::Info)
		.parse_default_env()
		.init();
	let config = std::fs::read_to_string("config.toml")
		.context("failed to read config.toml")
		.and_then(|file| toml::from_str::<Config>(&file).context("failed to parse config.toml"))?;
	if let Some(pin_cores) = &config.pin_cores {
		pin_cpus(pin_cores).context("failed to pin cores")?;
	}
	if config.lower_process_priority {
		lower_process_priority().context("failed to lower process priority")?;
	}
	let output_dir = config.output_dir();
	if !output_dir.exists() {
		std::fs::create_dir_all(output_dir).with_context(|| {
			format!(
				"failed to create directory {path}",
				path = output_dir.display()
			)
		})?;
	}
	let input_dir = &config.input_dir;
	info!("waiting for files in {path}", path = input_dir.display());
	loop {
		let dir = std::fs::read_dir(input_dir).with_context(|| {
			format!(
				"failed to read input dir {path}",
				path = input_dir.display()
			)
		})?;
		for entry in dir
			.filter_map(Result::ok)
			.filter(|entry| entry.path().is_file())
		{
			let path = entry.path();
			debug!("processing {path}", path = path.display());
			if let Err(err) = handle_file(&path, &config)
				.with_context(|| format!("failed to handle {path}", path = path.display()))
			{
				error!("failed to handle file: {err:?}");
			}
		}
		std::thread::sleep(Duration::from_secs(60));
	}
}

fn handle_file(path: &Path, config: &Config) -> Result<()> {
	if !check_file_writable(path).context("failed to check if input file is locked")? {
		debug!("file locked: {path}", path = path.display());
		return Ok(());
	}
	let filename = path
		.file_name()
		.context("failed to get filename")?
		.to_str()
		.context("failed to convert filename to string")?
		.to_owned();
	let out_path = config.output_dir().join(format!("{filename}.zst"));
	if out_path.exists() {
		debug!(
			"{path} already processed to {out_path}",
			path = path.display(),
			out_path = out_path.display()
		);
		return Ok(());
	}
	info!("compressing {path}", path = path.display());
	compress::compress_file(path, &out_path, config).with_context(|| {
		format!(
			"failed to compress {in_path} to {out_path}",
			in_path = path.display(),
			out_path = out_path.display()
		)
	})?;
	info!("finished compressing {path}", path = path.display());
	if config.delete_after {
		info!("removing original file at {path}", path = path.display());
		std::fs::remove_file(path).context("failed to remove original file")?;
	}
	Ok(())
}
