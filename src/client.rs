use codec;
use message::Message;
use message;
use error::{Error, ErrorKind};

use futures::{Future, Sink, Stream, Poll, StartSend, Async};

use tokio_core::io::{Io, Framed};
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};

use tokio_tls::{ConnectAsync, TlsConnectorExt, TlsStream};

use native_tls::TlsConnector;

use std::net::{SocketAddr, Shutdown};
use std::time;

const PING_TIMEOUT_IN_SECONDS: u64 = 10 * 60;
const COMMAND_PING: &'static str = "PING";

pub type IrcFramedStream<T> where T: Io = Framed<T, codec::IrcCodec>;

pub struct Client {
    host: SocketAddr,
}

impl Client {
    pub fn new<H: Into<SocketAddr>>(host: H) -> Client {
        Client { host: host.into() }
    }

    pub fn connect(&self, handle: &Handle) -> ClientConnectFuture {
        let tcp_stream = TcpStream::connect(&self.host, handle);

        ClientConnectFuture { inner: tcp_stream }
    }

    pub fn connect_tls<D: Into<String>>(&self,
                                        handle: &Handle,
                                        domain: D)
                                        -> ClientConnectTlsFuture {
        let tls_connector = match TlsConnector::builder() {
            Ok(tls_builder) => {
                match tls_builder.build() {
                    Ok(connector) => connector,
                    Err(err) => {
                        return ClientConnectTlsFuture::Err(ErrorKind::Tls(err).into());
                    }
                }
            }
            Err(err) => {
                return ClientConnectTlsFuture::Err(ErrorKind::Tls(err).into());
            }
        };

        let tcp_stream = TcpStream::connect(&self.host, handle);

        ClientConnectTlsFuture::TcpConnecting(tcp_stream, tls_connector, domain.into())
    }
}

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

pub enum ClientConnectTlsFuture {
    Err(Error),
    TcpConnecting(TcpStreamNew, TlsConnector, String),
    TlsHandshake(ConnectAsync<TcpStream>),
}

impl Future for ClientConnectTlsFuture {
    type Item = IrcTransport<TlsStream<TcpStream>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        let connect_async = match *self {
            ClientConnectTlsFuture::Err(ref mut error) => {
                let error = ::std::mem::replace(error, ErrorKind::Unexpected.into());
                return Err(error);
            }

            ClientConnectTlsFuture::TcpConnecting(ref mut tcp_connect_future,
                                                  ref mut tls_connector,
                                                  ref domain) => {
                let tcp_stream = try_ready!(tcp_connect_future.poll());

                tls_connector.connect_async(&domain, tcp_stream)
            }

            ClientConnectTlsFuture::TlsHandshake(ref mut tls_connect_future) => {
                let tls_stream = try_ready!(tls_connect_future.poll());

                return Ok(Async::Ready(IrcTransport::new(tls_stream.framed(codec::IrcCodec))));
            }
        };

        ::std::mem::replace(self, ClientConnectTlsFuture::TlsHandshake(connect_async));

        Ok(Async::NotReady)
    }
}

pub struct IrcTransport<T: Io> {
    inner: IrcFramedStream<T>,
    last_ping: time::Instant,
}

impl<T: Io> IrcTransport<T> {
    fn new(inner: IrcFramedStream<T>) -> IrcTransport<T> {
        IrcTransport {
            inner: inner,
            last_ping: time::Instant::now(),
        }
    }

    fn poll_next(&mut self) -> Poll<Option<Message>, Error> {
        loop {
            match try_ready!(self.inner.poll()) {
                Some(ref message) if message.command == COMMAND_PING => {
                    self.last_ping = time::Instant::now();

                    if let Some(ref suffix) = message.suffix {
                        let result = self.inner.start_send(message::pong(suffix.as_str()))?;

                        assert!(result.is_ready());

                        self.inner.poll_complete()?;
                    }
                }
                message => return Ok(Async::Ready(message)),
            }
        }
    }
}

impl Stream for IrcTransport<TcpStream> {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let elapsed = self.last_ping.elapsed();

        if elapsed.as_secs() >= PING_TIMEOUT_IN_SECONDS {
            self.inner.get_mut().shutdown(Shutdown::Both)?;
            return Err(ErrorKind::ConnectionReset.into());
        }

        self.poll_next()
    }
}

impl Stream for IrcTransport<TlsStream<TcpStream>> {
    type Item = Message;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let elapsed = self.last_ping.elapsed();

        if elapsed.as_secs() >= PING_TIMEOUT_IN_SECONDS {
            self.inner.get_mut().get_mut().shutdown()?;
            return Err(ErrorKind::ConnectionReset.into());
        }

        self.poll_next()
    }
}

impl<T: Io> Sink for IrcTransport<T> {
    type SinkItem = Message;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.inner.start_send(item)?)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
    }
}
