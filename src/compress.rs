// SPDX-License-Identifier: Zlib
use crate::Config;
use color_eyre::eyre::{Result, WrapErr};
use std::{
	fs::File,
	io::{BufReader, BufWriter, Write},
	path::Path,
};
use zstd::stream::write::Encoder as ZstdEncoder;

pub fn compress_file(
	in_path: impl AsRef<Path>,
	out_path: impl AsRef<Path>,
	config: &Config,
) -> Result<()> {
	let in_path = in_path.as_ref();
	let out_path = out_path.as_ref();
	let size: u64 = in_path
		.metadata()
		.context("failed to get input file metadata")?
		.len();
	let mut in_file = File::open(in_path)
		.map(BufReader::new)
		.context("failed to open input file")?;

	// setup and compress
	let mut encoder =
		create_encoder(out_path, size, config).context("failed to setup zstd encoder")?;
	std::io::copy(&mut in_file, &mut encoder).context("failed to copy data to zstd compressor")?;

	// finish up everything
	let mut out_writer = encoder
		.try_finish()
		.map_err(|(_, err)| err)
		.context("failed to finalize compression")?;
	out_writer
		.flush()
		.context("failed to flush output buffer")?;
	out_writer
		.into_inner()?
		.sync_all()
		.context("failed to sync output file to disk")?;
	Ok(())
}

fn create_encoder(
	out_path: impl AsRef<Path>,
	size: u64,
	config: &Config,
) -> Result<ZstdEncoder<'static, BufWriter<File>>> {
	let file = File::create(out_path.as_ref())
		.map(BufWriter::new)
		.context("failed to create output file")?;
	let mut encoder = ZstdEncoder::new(file, config.compression_level)
		.context("failed to create zstd encoder")?;
	encoder
		.set_pledged_src_size(Some(size))
		.context("failed to set pledged src size of zstd encoder")?;
	encoder
		.include_contentsize(true)
		.context("failed to include content size for zstd encoder")?;
	encoder
		.long_distance_matching(true)
		.context("failed to enable long-distance matching for zstd encoder")?;
	let workers = config.zstd_workers();
	encoder
		.multithread(workers)
		.with_context(|| format!("failed to set zstd encoder workers to {workers}"))?;
	Ok(encoder)
}
