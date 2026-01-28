use super::*;

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{future::FutureExt, pin_mut, AsyncRead, AsyncWrite};
use hickory_resolver::{
    config::{NameServerConfig, ResolverOpts},
    name_server::{ConnectionProvider, GenericConnector},
    proto::{
        runtime::{Executor, RuntimeProvider, Spawn, Time},
        tcp::DnsTcpStream,
        udp::{DnsUdpSocket, UdpSocket},
        ProtoError,
    },
    Resolver,
};
use socket2::{Domain, Protocol, Socket, Type};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub type AsyncStdResolver = Resolver<AsyncStdConnectionProvider>;

#[derive(Clone, Default)]
pub struct AsyncStdHandle {}

impl Spawn for AsyncStdHandle {
    fn spawn_bg<F>(&mut self, future: F)
    where
        F: Future<Output = Result<(), ProtoError>> + Send + 'static,
    {
        spawn("hickory-resolver task", future).detach();
    }
}

#[derive(Clone, Default)]
pub struct AsyncStdRuntimeProvider(AsyncStdHandle);

impl AsyncStdRuntimeProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RuntimeProvider for AsyncStdRuntimeProvider {
    type Handle = AsyncStdHandle;
    type Timer = AsyncStdTime;
    type Udp = AsyncStdUdpSocket;
    type Tcp = AsyncStdTcpStream;

    fn create_handle(&self) -> Self::Handle {
        self.0.clone()
    }

    fn connect_tcp(
        &self,
        server_addr: SocketAddr,
        bind_addr: Option<SocketAddr>,
        wait_for: Option<Duration>,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
        Box::pin(async move {
            let stream = match bind_addr {
                Some(bind_addr) => {
                    let future = async_std::task::spawn_blocking(move || {
                        let domain = match bind_addr {
                            SocketAddr::V4(_) => Domain::IPV4,
                            SocketAddr::V6(_) => Domain::IPV6,
                        };
                        let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
                        socket.bind(&bind_addr.into())?;
                        socket.connect(&server_addr.into())?;
                        let std_stream: std::net::TcpStream = socket.into();
                        let stream = async_std::net::TcpStream::from(std_stream);
                        Ok::<_, io::Error>(stream)
                    });
                    let wait_for = wait_for.unwrap_or(CONNECT_TIMEOUT);
                    async_std::io::timeout(wait_for, future).await?
                }
                None => {
                    let future = async_std::net::TcpStream::connect(server_addr);
                    let wait_for = wait_for.unwrap_or(CONNECT_TIMEOUT);
                    async_std::io::timeout(wait_for, future).await?
                }
            };
            stream.set_nodelay(true)?;
            Ok(AsyncStdTcpStream(stream))
        })
    }

    fn bind_udp(
        &self,
        local_addr: SocketAddr,
        _server_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
        Box::pin(async move {
            async_std::net::UdpSocket::bind(local_addr)
                .await
                .map(AsyncStdUdpSocket)
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AsyncStdTime;

impl Time for AsyncStdTime {
    fn delay_for<'a>(duration: Duration) -> Pin<Box<dyn Send + Future<Output = ()> + 'a>> {
        Box::pin(async move { async_std::task::sleep(duration).await })
    }

    fn timeout<'a, F>(
        duration: Duration,
        future: F,
    ) -> Pin<Box<dyn Send + Future<Output = Result<F::Output, std::io::Error>> + 'a>>
    where
        F: 'static + Future + Send,
    {
        Box::pin(
            async move { async_std::io::timeout(duration, async move { Ok(future.await) }).await },
        )
    }
}

#[derive(Clone, Default)]
pub struct AsyncStdConnectionProvider {
    connection_provider: GenericConnector<AsyncStdRuntimeProvider>,
}

impl Executor for AsyncStdConnectionProvider {
    fn new() -> Self {
        let p = AsyncStdRuntimeProvider::new();
        Self {
            connection_provider: GenericConnector::new(p),
        }
    }

    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        async_std::task::block_on(future)
    }
}

impl ConnectionProvider for AsyncStdConnectionProvider {
    type Conn = <GenericConnector<AsyncStdRuntimeProvider> as ConnectionProvider>::Conn;
    type FutureConn = <GenericConnector<AsyncStdRuntimeProvider> as ConnectionProvider>::FutureConn;
    type RuntimeProvider = AsyncStdRuntimeProvider;

    fn new_connection(
        &self,
        config: &NameServerConfig,
        options: &ResolverOpts,
    ) -> Result<Self::FutureConn, std::io::Error> {
        self.connection_provider.new_connection(config, options)
    }
}

pub struct AsyncStdUdpSocket(async_std::net::UdpSocket);

impl DnsUdpSocket for AsyncStdUdpSocket {
    type Time = AsyncStdTime;
    fn poll_recv_from(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<(usize, SocketAddr)>> {
        let fut = self.0.recv_from(buf);
        pin_mut!(fut);

        fut.poll_unpin(cx)
    }

    fn poll_send_to(
        &self,
        cx: &mut Context<'_>,
        buf: &[u8],
        target: SocketAddr,
    ) -> Poll<io::Result<usize>> {
        let fut = self.0.send_to(buf, target);
        pin_mut!(fut);

        fut.poll_unpin(cx)
    }
}

impl UdpSocket for AsyncStdUdpSocket {
    fn connect<'a>(
        addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self>> + 'a>> {
        Box::pin(async move {
            let bind_addr: SocketAddr = match addr {
                SocketAddr::V4(_addr) => (Ipv4Addr::UNSPECIFIED, 0).into(),
                SocketAddr::V6(_addr) => (Ipv6Addr::UNSPECIFIED, 0).into(),
            };

            Self::connect_with_bind(addr, bind_addr).await
        })
    }

    fn connect_with_bind<'a>(
        _addr: SocketAddr,
        bind_addr: SocketAddr,
    ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self>> + 'a>> {
        Box::pin(async move {
            let socket = async_std::net::UdpSocket::bind(bind_addr).await?;

            // TODO: research connect more, it appears to break receive tests on UDP
            // socket.connect(addr).await?;
            Ok(Self(socket))
        })
    }

    fn bind<'a>(addr: SocketAddr) -> Pin<Box<dyn Send + Future<Output = io::Result<Self>> + 'a>> {
        Box::pin(async move {
            async_std::net::UdpSocket::bind(addr)
                .await
                .map(AsyncStdUdpSocket)
        })
    }
}

pub struct AsyncStdTcpStream(async_std::net::TcpStream);

impl DnsTcpStream for AsyncStdTcpStream {
    type Time = AsyncStdTime;
}

impl AsyncWrite for AsyncStdTcpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_write(cx, bytes)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl AsyncRead for AsyncStdTcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &mut [u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
        Pin::new(&mut self.0).poll_read(cx, bytes)
    }
}
