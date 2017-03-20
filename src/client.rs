use codec;
use message::Message;
use commands::Ping;
use commands;
use error::{Error, ErrorKind};

use futures::{Future, Sink, Stream, Poll, StartSend, Async};

use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;

use tokio_tls::{ConnectAsync, TlsConnectorExt, TlsStream};

use native_tls::TlsConnector;

use std::net::SocketAddr;
use std::time;

const PING_TIMEOUT_IN_SECONDS: u64 = 10 * 60;

pub type IrcFramedStream<T> where T: AsyncRead + AsyncWrite = Framed<T, codec::IrcCodec>;

/// A light-weight client type for establishing connections to remote servers.
/// This type consumes a given `SocketAddr` and provides several methods for
/// establishing connections to a remote server.  Currently these methods
/// allow for the connection to a server with unencrypted data and TLS
/// encrypted data.
///
/// Each of the connection methods will return a future, that when successfully
/// resolved, will provide a `Stream` that allows for communication with the
/// remote server.
pub struct Client {
    host: SocketAddr,
}

impl Client {
    /// Create a new instance of `Client` that provides the ability to establish
    /// remote server connections with the specified host.
    pub fn new<H: Into<SocketAddr>>(host: H) -> Client {
        Client { host: host.into() }
    }

    /// Returns a future, that when resolved provides an unecrypted `Stream`
    /// that can be used to receive `Message` from the server and send `Message`
    /// to the server.
    ///
    /// The resulting `Stream` can be `split` into a separate `Stream` for
    /// receiving `Message` from the server and a `Sink` for sending `Message`
    /// to the server.
    pub fn connect(&self, handle: &Handle) -> ClientConnectFuture {
        let tcp_stream = TcpStream::connect(&self.host, handle);

        ClientConnectFuture { inner: tcp_stream }
    }

    /// Returns a future, that when resolved provides a TLS encrypted `Stream`
    /// that can be used to receive `Message` from the server and send `Message`
    /// to the server.
    ///
    /// The resulting `Stream` can be `split` into a separate `Stream` for
    /// receiving `Message` from the server and a `Sink` for sending `Message`
    /// to the server.
    ///
    /// `domain` is the domain name of the remote server being connected to.
    /// it is required to validate the security of the connection.
    pub fn connect_tls<D: Into<String>>(&self,
                                        handle: &Handle,
                                        domain: D)
                                        -> ClientConnectTlsFuture {
        use self::ClientConnectTlsFuture::*;

        let tls_connector = match TlsConnector::builder() {
            Ok(tls_builder) => {
                match tls_builder.build() {
                    Ok(connector) => connector,
                    Err(err) => {
                        return TlsErr(ErrorKind::Tls(err).into());
                    }
                }
            }
            Err(err) => {
                return TlsErr(ErrorKind::Tls(err).into());
            }
        };

        let tcp_stream = TcpStream::connect(&self.host, handle);

        TcpConnecting(tcp_stream, tls_connector, domain.into())
    }
}

/// Represents a future, that when resolved provides an unecrypted `Stream`
/// that can be used to receive `Message` from the server and send `Message`
/// to the server.
pub struct ClientConnectFuture {
    inner: TcpStreamNew,
}

impl Future for ClientConnectFuture {
    type Item = IrcTransport<TcpStream>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let framed: IrcFramedStream<_> = try_ready!(self.inner.poll()).framed(codec::IrcCodec);
        let irc_transport = IrcTransport::new(framed);

        Ok(Async::Ready(irc_transport))
    }
}

/// Represents a future, that when resolved provides a TLS encrypted `Stream`
/// that can be used to receive `Message` from the server and send `Message`
/// to the server.
pub enum ClientConnectTlsFuture {
    TlsErr(Error),
    TcpConnecting(TcpStreamNew, TlsConnector, String),
    TlsHandshake(ConnectAsync<TcpStream>),
}

// This future is represented internally as a simple state machine.
// The state machine can either be in error, waiting for a TCP connection to
// fully resolve or error out, or waiting for a TLS handshake to fully resolve
// or error out.  The various error types are all converted to this crate's
// own error representation.
//
// The typical transition is that this future will first resolve an open TCP
// socket, which is then used to establish a TLS connection via a handshake
// to the remote server. If at any point any of these futures fail to resolve
// an error is produced by this future.
//
// Due to the way the the underlying TLS library works, which requires a
// `TlsConnector` to be created, an operation that can possibly fail, this
// future may start in an error state and will immediately resolve with that
// error on the next call to `poll`.
impl Future for ClientConnectTlsFuture {
    type Item = IrcTransport<TlsStream<TcpStream>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::ClientConnectTlsFuture::*;

        let connect_async = match *self {
            TlsErr(ref mut error) => {
                let error = ::std::mem::replace(error, ErrorKind::Unexpected.into());
                return Err(error);
            }

            TlsHandshake(ref mut tls_connect_future) => {
                let framed = try_ready!(tls_connect_future.poll()).framed(codec::IrcCodec);
                let irc_transport = IrcTransport::new(framed);

                return Ok(Async::Ready(irc_transport));
            }

            TcpConnecting(ref mut tcp_connect_future, ref mut tls_connector, ref domain) => {

                let tcp_stream = try_ready!(tcp_connect_future.poll());
                tls_connector.connect_async(&domain, tcp_stream)
            }
        };

        *self = ClientConnectTlsFuture::TlsHandshake(connect_async);

        Ok(Async::NotReady)
    }
}

/// `IrcTransport` represents a framed IRC stream returned from the connection
/// methods when their given futures are resolved. It internally handles the
/// processing of PING requests and timing out the connection when no PINGs
/// have been recently received from the server.
///
/// It is possible to split `IrcTransport` into `Stream` and `Sink` via the
/// the `split` method.
pub struct IrcTransport<T: AsyncRead + AsyncWrite> {
    inner: IrcFramedStream<T>,
    last_ping: time::Instant,
}

impl<T: AsyncRead + AsyncWrite> IrcTransport<T> {
    fn new(inner: IrcFramedStream<T>) -> IrcTransport<T> {
        IrcTransport {
            inner: inner,
            last_ping: time::Instant::now(),
        }
    }

    fn ping_timed_out(&self) -> bool {
        self.last_ping.elapsed().as_secs() >= PING_TIMEOUT_IN_SECONDS
    }

    fn poll_next(&mut self) -> Poll<Option<Message>, Error> {
        loop {
            if let Some(message) = try_ready!(self.inner.poll()) {
                if let Some(Ping(host)) = message.command::<Ping>() {
                    self.last_ping = time::Instant::now();
                    let result = self.inner.start_send(commands::pong(host)?)?;

                    assert!(result.is_ready());

                    self.inner.poll_complete()?;
                    continue;
                }

                return Ok(Async::Ready(Some(message)));
            }

            return Ok(Async::Ready(None));
        }
    }
}

impl Stream for IrcTransport<TcpStream> {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.ping_timed_out() {
            self.inner.close()?;
            return Err(ErrorKind::ConnectionReset.into());
        }

        self.poll_next()
    }
}

impl Stream for IrcTransport<TlsStream<TcpStream>> {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.ping_timed_out() {
            self.inner.close()?;
            return Err(ErrorKind::ConnectionReset.into());
        }

        self.poll_next()
    }
}

impl<T: AsyncRead + AsyncWrite> Sink for IrcTransport<T> {
    type SinkItem = Message;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.inner.start_send(item)?)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
    }
}
