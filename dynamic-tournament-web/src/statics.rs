use std::mem::MaybeUninit;

use crate::Config;

static mut CONFIG: MaybeUninit<Config> = MaybeUninit::uninit();

#[inline]
pub fn config() -> &'static Config {
    // SAFETY: `set_config` must have been at the start of the program.
    // CONFIG is initialzed.
    unsafe { CONFIG.assume_init_ref() }
}

/// Sets the config. You must call this function only once during the lifetime of the program.
///
/// # Safety
///
/// While this function executes there must be no references to the config. A reference can be
/// obtained by calling [`config`]. You must also only call this function once in the lifetime of
/// the program.
#[inline]
pub(super) unsafe fn set_config(config: Config) {
    CONFIG.write(config);
}
