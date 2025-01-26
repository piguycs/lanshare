#![feature(let_chains)]

pub mod client;
pub mod error;
pub mod handler;
pub mod server;

#[cfg(test)]
mod test_utils;

pub const BC_CFG: bincode::config::Configuration = bincode::config::standard();


pub mod reexports {
  pub use quinn;
  pub use quinn::rustls;
}
