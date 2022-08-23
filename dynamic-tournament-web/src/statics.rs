use std::mem::MaybeUninit;

use serde::{Deserialize, Serialize};

/// The global config.
///
/// This instance always lives for the lifetime of the program.
///
/// # Safety
///
/// An `Config` instance is always expected to have a `'static` lifetime. Some methods make use of
/// this assumption to provide safe methods.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // We never need to resize this, so `Box<str>` saves us 1 * usize of space.
    pub api_base: Box<str>,
    pub root: Box<str>,
}

impl Config {
    /// Returns the defined api_base.
    #[inline]
    pub fn api_base(&self) -> &'static str {
        &self.static_ref().api_base
    }

    #[inline]
    pub fn root(&self) -> &'static str {
        &self.static_ref().root
    }

    /// Converts a `&Config` reference into a `&'static Config` reference.
    #[inline]
    fn static_ref(&self) -> &'static Self {
        // SAFETY: The caller must guarantee that `self` has a `'static` lifetime.
        unsafe { std::mem::transmute(self) }
    }
}

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
