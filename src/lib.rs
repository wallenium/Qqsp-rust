//! Rust-Neuimplementierung der QSP-Engine (libqsp 5.7.0).
mod api;
mod code;
mod cp1251;
mod crc;
mod engine;
mod errors;
mod ext59;
mod exec;
mod expr;
mod ffi;
mod gamefile;
mod host;
mod text;
mod value;
mod vars;
mod world;

pub use api::ErrorInfo;
pub use ffi::QspHost;
pub use engine::{Engine, QSP_VER};
pub use host::{Host, NullHost, Win};
pub use value::Val;
pub use errors::describe as error_desc;
