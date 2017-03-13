extern crate tokio_irc_client;
extern crate futures;
extern crate tokio_core;

use std::net::ToSocketAddrs;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Stream;
use futures::Sink;

use tokio_irc_client::Client;
use tokio_irc_client::message;

fn main() {
    // Create the event loop
    let mut ev = Core::new().unwrap();
    let handle = ev.handle();

    // Do a DNS query and get the first socket address for Freenode
    let addr = "irc.freenode.org:6667".to_socket_addrs().unwrap().next().unwrap();

    // Create the client future and connect to the server
    // In order to connect we need to send a NICK message,
    // followed by a USER message
    let client = Client::new(addr)
        .connect(&handle).and_then(|irc| {
            irc.send(message::nick("RustBot"))
        }).and_then(|irc| {
            irc.send(message::user("RustBot", "Example bot written in Rust"))
        }).and_then(|irc| {
            // After that, we can send the JOIN message to join a channel and
            // start getting messages from it. The library handles the PINGs
            // for us so we can stay connected
            irc.send(message::join("#programming"))
        }).and_then(|irc| {
            // We iterate over the IRC connection, giving us all the packets
            // Checking if the command is PRIVMSG allows us to print just the
            // messages
            irc.for_each(|packet| {
                if packet.command == "PRIVMSG" {
                    let prefix = packet.prefix.unwrap().to_owned();
                    let nick = prefix.split("!").next().unwrap();
                    println!("<{}> {}", nick, packet.suffix.unwrap());
                }
                Ok(())
            })
        });

    ev.run(client).unwrap();
}
