// SPDX-License-Identifier: Zlib
use color_eyre::eyre::{Result, WrapErr};
use std::{os::windows::ffi::OsStrExt, path::Path};
use windows::{
	core::PCWSTR,
	Win32::{
		Foundation::{CloseHandle, ERROR_SHARING_VIOLATION, GENERIC_READ, HANDLE},
		Storage::FileSystem::{
			CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING,
		},
	},
};

pub fn check_file_writable(path: impl AsRef<Path>) -> Result<bool> {
	let wide_path: Vec<u16> = path
		.as_ref()
		.as_os_str()
		.encode_wide()
		.chain(Some(0))
		.collect();

	// If that failed, try to open it with shared access to confirm it exists
	match unsafe {
		CreateFileW(
			PCWSTR::from_raw(wide_path.as_ptr()),
			GENERIC_READ.0,
			FILE_SHARE_READ,
			None,
			OPEN_EXISTING,
			FILE_FLAGS_AND_ATTRIBUTES::default(),
			HANDLE::default(),
		)
	} {
		Ok(handle) => {
			if handle.is_invalid() {
				Ok(false)
			} else {
				unsafe {
					CloseHandle(handle).context("failed to close file handle")?;
				}
				Ok(true)
			}
		}
		Err(err) if err.code() == ERROR_SHARING_VIOLATION.to_hresult() => Ok(false),
		Err(err) => {
			let reason = format!("CreateFileW failed (err code {})", err.code());
			Err(err).context(reason)
		}
	}
}
