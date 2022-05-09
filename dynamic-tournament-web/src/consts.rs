//! Compile-tile constants for configuration
use crate::Mountpoint;

/// The mountpoint of the application. [`Mountpoint::Body`] will start the application at the
/// `<body>` html tag. [`Mountpoint::Element`] will start the application at the html tag with
/// the given id.
///
/// # Examples
///
/// Mount and start the application at the `<body>` html tag.
/// ```
/// pub const MOUNTPOINT: Mountpoint = Mountpoint::Body;
/// ```
///
/// Mount and start the application at the `<div id="main">` html tag.
/// ```
/// pub const MOUNTPOINT: Mountpoint = Mountpoint::Element("main");
/// ```
pub const MOUNTPOINT: Mountpoint = Mountpoint::Body;

pub const TITLE_BASE: &str = "Hardstuck Tournaments";
