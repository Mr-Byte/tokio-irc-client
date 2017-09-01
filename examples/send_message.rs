extern crate futures;
extern crate pircolate;
extern crate tokio_core;
extern crate tokio_irc_client;

use std::net::ToSocketAddrs;
use std::str::FromStr;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Sink;
use futures::Stream;
use futures::stream;

use tokio_irc_client::Client;
use pircolate::message;
use pircolate::command::Welcome;

fn main() {
    // Create the event loop
    let mut ev = Core::new().unwrap();
    let handle = ev.handle();

    let mut server = "irc.freenode.org:6667".to_string();
    if let Ok(env_override) = std::env::var("IRC_SERVER") {
        server = env_override;
    }

    // Do a DNS query and get the first socket address for Freenode
    let addr = server.to_socket_addrs().unwrap().next().unwrap();

    // Create the client future and connect to the server
    // In order to connect we need to send a NICK message,
    // followed by a USER message
    let client = Client::new(addr)
        .connect(&handle)
        .and_then(|irc| {
            let connect_sequence = vec![
                message::client::nick("RustBot2"),
                message::client::user("RustBot2", "Example bot written in Rust"),
            ];

            irc.send_all(stream::iter(connect_sequence))
        })
        .and_then(|(irc, _)| {
            let (send, recv) = irc.split();

            let welcome_received = recv.skip_while(|msg| {
                match msg.command() {
                    Some(Welcome(_, _)) => Ok(false),
                    _ => Ok(true), // continue waiting for a welcome
                }
            });

            welcome_received
                .into_future()
                .map_err(|res| res.0)
                .and_then(|(_, recv)| Ok((send, recv)))
        })
        .and_then(|(send, recv)| {
            send.send(message::client::join("#tokio-irc", None).unwrap())
                .and_then(|send| Ok((send, recv)))
        })
        .and_then(|(send, recv)| {
            send.send_all(stream::iter(vec![
                message::client::priv_msg("#tokio-irc", "Hello World!"),
                message::client::priv_msg("#tokio-irc", "Goodbye world"),
            ])).and_then(|(send, _)| Ok((send, recv)))
        })
        .and_then(|(send, recv)| {
            send.send(
                message::Message::from_str("PART #tokio-irc :are you still there\r\n").unwrap(),
            ).and_then(|send| Ok((send, recv)))
        })
        .and_then(|(send, _)| {
            send.send(message::Message::from_str("QUIT").unwrap())
        });

    ev.run(client).unwrap();
}
