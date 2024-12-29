// SPDX-License-Identifier: Zlib
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	/// The directory to watch and compress files from.
	pub input_dir: PathBuf,
	/// The directory to output compressed files to.
	/// If unspecified, it will default to the input director
	pub output_dir: Option<PathBuf>,
	/// If specified, the process's affinity will be set to the given
	/// cores.
	pub pin_cores: Option<Vec<usize>>,
	/// Whether to lower the process's priority class or not.
	/// Defaults to true, to minimize the chance of interfering with any other
	/// processes or servers.
	#[serde(default = "default_true")]
	pub lower_process_priority: bool,
	/// The zstd compression level to use.
	/// Defaults to [`zstd::DEFAULT_COMPRESSION_LEVEL`], which is 3.
	#[serde(default = "default_compression_level")]
	pub compression_level: i32,
	/// Whether to delete the original file after a successful compression.
	/// Defaults to true.
	#[serde(default = "default_true")]
	pub delete_after: bool,
}

impl Config {
	/// Returns the output directory, or if unspecified, the input directory.
	pub fn output_dir(&self) -> &Path {
		self.output_dir.as_deref().unwrap_or(&self.input_dir)
	}

	/// Returns how many worker threads should be used for zstd compression.
	pub fn zstd_workers(&self) -> u32 {
		self.pin_cores
			.as_ref()
			.map(|cores| cores.len())
			.unwrap_or(1) as u32
	}
}

const fn default_compression_level() -> i32 {
	zstd::DEFAULT_COMPRESSION_LEVEL
}

const fn default_true() -> bool {
	true
}
