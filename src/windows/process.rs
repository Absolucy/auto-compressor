// SPDX-License-Identifier: Zlib
use color_eyre::eyre::{eyre, Result, WrapErr};
use windows::Win32::System::Threading::{
	GetCurrentProcess, SetPriorityClass, SetProcessAffinityMask, BELOW_NORMAL_PRIORITY_CLASS,
};

pub fn pin_cpus(cores: &[usize]) -> Result<()> {
	if cores.is_empty() {
		return Err(eyre!("Must specify at least 1 cpu core to pin."));
	}
	unsafe {
		SetProcessAffinityMask(
			GetCurrentProcess(),
			cores.iter().fold(0_usize, |mask, &core| mask | (1 << core)),
		)
		.context("failed to set process affinity")
	}
}

pub fn lower_process_priority() -> Result<()> {
	unsafe {
		SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS)
			.context("failed to set process priority class to BELOW_NORMAL_PRIORITY_CLASS")
	}
}
