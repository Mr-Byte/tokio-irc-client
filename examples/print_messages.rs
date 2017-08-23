extern crate tokio_irc_client;
extern crate futures;
extern crate tokio_core;
extern crate pircolate;

use std::net::ToSocketAddrs;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Stream;
use futures::Sink;
use futures::stream;

use tokio_irc_client::Client;
use pircolate::message;
use pircolate::command::PrivMsg;

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
                message::client::nick("RustBot"),
                message::client::user("RustBot", "Example bot written in Rust"),
                message::client::join("#tokio-irc", None),
            ];

            irc.send_all(stream::iter(connect_sequence))
        })
        .and_then(|(irc, _)| {
            // We iterate over the IRC connection, giving us all the packets
            // Checking if the command is PRIVMSG allows us to print just the
            // messages
            irc.for_each(|incoming_message| {
                if let Some(PrivMsg(_, message)) = incoming_message.command::<PrivMsg>() {
                    if let Some((nick, _, _)) = incoming_message.prefix() {
                        println!("<{}> {}", nick, message)
                    }
                }

                Ok(())
            })
        });

    ev.run(client).unwrap();
}
