//! Provide message both send and receive
//! **only used in projects of the `dvorak` -- who as 'me' for practice**
//!
//! # Usage
//!
//! use full features
//!
//! dvorak_message = { version = "*", features = ["full"]}
//!
//!
//! or particular feature
//!
//! dvorak_message = { version = "*", features = ["message"]}
//!
//!
//! # Features
//! - message: wrap the data into Massage, and allow both send and receive

#[cfg(feature = "message")]
pub mod message;
