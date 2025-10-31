//! HTTP request handlers

pub mod health;

pub use health::{health, live, ready, version};
