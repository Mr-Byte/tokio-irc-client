//! Low-level futures based IRC client library for Rust
//!
//! Tihs library is an implementation of the IRCv3 protocol which uses futures
//! to provide asynchronous processing of incoming and outgoing messages between
//! the client and the server.
//!
//! This library makes extensive use of tokio and futures to provide
//! asynchronous handling of IRC via the `Future`, `Stream`, and `Sink` traits.
//! This allows for the usage of combinators on the stream of incoming messages
//! to allow for easy processing of messages.
//!
//! The abstraction provided by this library is currently very minimal.
//! It currently handles connecting to a remote IRC server either through
//! unencypted sockets or TLS encrypted sockets. It internally handles the
//! parsing of IRC messages and has several helper functions to build IRC
//! messages to be sent to the server. It also handles responding to PING
//! requests from the server and will timeout the connection if no PINGs are
//! received after a certain duration (currently 10 minutes).
//!
//! The main type in this library is the `Cient` struct, which provides the
//! ability to connect to a remote host. The various connection methods on this
//! type return a future, that when complete, provides a stream of IRC messages
//! that can also be written to.  It is possible to call the `split` function
//! on this stream to get a `Stream` or incoming IRC messages and `Sink` for
//! for sending messages to the server.
//!
//! Currently there are no samples of tutorials available, but will be provided
//! eventually.

#[macro_use]
extern crate futures;
#[macro_use]
extern crate error_chain;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_tls;
extern crate bytes;
extern crate native_tls;

mod codec;
pub mod error;
pub mod client;
pub mod message;

pub use client::{Client, ClientConnectFuture, ClientConnectTlsFuture};
pub use error::Error;