//! Platform specific utilities are stored in here
//!
//! ## Linux
//! A D-Bus daemon is constructed using the zbus crate. Furthermore, the TUN device controller is
//! directly owned by the LinuxPlatform struct

#[cfg(target_os = "linux")]
pub mod linux;
