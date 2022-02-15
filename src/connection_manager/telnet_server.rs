extern crate futures;
extern crate tokio;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;


use ansi_term::Colour;
use ansi_term::Style;
use anyhow;
use futures::Future;
use tokio::net::tcp::WriteHalf;
use crate::world::states::ScreenType;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};



#[derive(Clone)]
pub struct TelnetServer {
    clients: Arc<Mutex<HashMap<(usize, ChannelId), thrussh::server::Handle>>>,
    id: usize,
    tx_data_channel: Arc<Mutex<Sender<String>>>,
    tx_command_channel: Arc<Mutex<Sender<WriteHalf>>>, // TODO use a type for commands
    command_buffer: String,
}

#[derive(Debug, Error)]
pub enum Error {

    /// The protocol is in an inconsistent state.
    #[error("Inconsistent state of the protocol")]
    Inconsistent,

    /// Index out of bounds.
    #[error("Index out of bounds")]
    IndexOutOfBounds,

    /// Message received/sent on unopened channel.
    #[error("Channel not open")]
    WrongChannel,

    /// Disconnected
    #[error("Disconnected")]
    Disconnect,

    /// Connection closed by the remote side.
    #[error("Connection closed by the remote side")]
    HUP,

    /// Connection timeout.
    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Channel send error")]
    SendError,

    #[error("Pending buffer limit reached")]
    Pending,

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Elapsed(#[from] tokio::time::error::Elapsed),
}

/// Server handler. Each client will have their own handler.
pub trait Handler: Sized {
    type Error: From<Error> + Send;

    /// The type of units returned by some parts of this handler.
    type FutureUnit: Future<Output = Result<(Self, Session), Self::Error>> + Send;

    /// The type of future bools returned by some parts of this handler.
    type FutureBool: Future<Output = Result<(Self, Session, bool), Self::Error>> + Send;

    /// Convert a `bool` to `Self::FutureBool`. This is used to
    /// produce the default handlers.
    fn finished_bool(self, b: bool, session: Session) -> Self::FutureBool;

    /// Produce a `Self::FutureUnit`. This is used to produce the
    /// default handlers.
    fn finished(self, session: Session) -> Self::FutureUnit;

    /// Called when the client closes a channel.
    #[allow(unused_variables)]
    fn channel_close(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        self.finished(session)
    }

    /// Called when the client sends EOF to a channel.
    #[allow(unused_variables)]
    fn channel_eof(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        self.finished(session)
    }

    /// Called when a new session channel is created.
    #[allow(unused_variables)]
    fn channel_open_session(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        self.finished(session)
    }

    /// Called when a new channel is created.
    #[allow(unused_variables)]
    fn channel_open_direct_tcpip(
        self,
        channel: ChannelId,
        host_to_connect: &str,
        port_to_connect: u32,
        originator_address: &str,
        originator_port: u32,
        session: Session,
    ) -> Self::FutureUnit {
        self.finished(session)
    }

    /// Called when a data packet is received. A response can be
    /// written to the `response` argument.
    #[allow(unused_variables)]
    fn data(self, channel: ChannelId, data: &[u8], session: Session) -> Self::FutureUnit {
        self.finished(session)
    }


    /// Called when the network window is adjusted, meaning that we
    /// can send more bytes.
    #[allow(unused_variables)]
    fn window_adjusted(
        self,
        channel: ChannelId,
        new_window_size: usize,
        mut session: Session,
    ) -> Self::FutureUnit {
        if let Some(ref mut enc) = session.common.encrypted {
            enc.flush_pending(channel);
        }
        self.finished(session)
    }

    /// Called when this server adjusts the network window. Return the
    /// next target window.
    #[allow(unused_variables)]
    fn adjust_window(&mut self, channel: ChannelId, current: u32) -> u32 {
        current
    }

   
    /// The client's pseudo-terminal window size has changed.
    #[allow(unused_variables)]
    fn window_change_request(
        self,
        channel: ChannelId,
        col_width: u32,
        row_height: u32,
        pix_width: u32,
        pix_height: u32,
        session: Session,
    ) -> Self::FutureUnit {
        self.finished(session)
    }

    /// The client is sending a signal (usually to pass to the
    /// currently running process).
    #[allow(unused_variables)]
    fn signal(self, channel: ChannelId, signal_name: Sig, session: Session) -> Self::FutureUnit {
        self.finished(session)
    }

    /// Used for reverse-forwarding ports, see
    /// [RFC4254](https://tools.ietf.org/html/rfc4254#section-7).
    #[allow(unused_variables)]
    fn tcpip_forward(self, address: &str, port: u32, session: Session) -> Self::FutureBool {
        self.finished_bool(false, session)
    }
    /// Used to stop the reverse-forwarding of a port, see
    /// [RFC4254](https://tools.ietf.org/html/rfc4254#section-7).
    #[allow(unused_variables)]
    fn cancel_tcpip_forward(self, address: &str, port: u32, session: Session) -> Self::FutureBool {
        self.finished_bool(false, session)
    }
}


pub trait Server {
    /// The type of handlers.
    type Handler: Handler + Send;
    /// Called when a new client connects.
    fn new(&mut self, peer_addr: Option<std::net::SocketAddr>) -> Self::Handler;
}

impl Server for TelnetServer {
    type Handler = Self;
    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        self.command_buffer.clear();
        s
    }
}


#[derive(Debug)]
/// Configuration of a server.
pub struct Config {
    /// The banner, usually a warning message shown to the client.
    pub banner: Option<&'static str>,
    /// The initial size of a channel (used for flow control).
    pub window_size: u32,
    /// The maximal size of a single packet.
    pub maximum_packet_size: u32,
    /// Time after which the connection is garbage-collected.
    pub connection_timeout: Option<std::time::Duration>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            banner: None,
            window_size: 2097152,
            maximum_packet_size: 32768,
            connection_timeout: Some(std::time::Duration::from_secs(600)),
        }
    }
}


pub fn init_telnet_server() -> (TelnetServer, Arc<Config>,
    Receiver<String>, Receiver<String>) {
// Configure the server

let mut config = Config::default();
config.connection_timeout = Some(std::time::Duration::from_secs(600));
config.banner = Some("Banner Test\n");
let config = Arc::new(config);

// The data channel: The channel players use to send actions etc....
let (data_tx, data_rx) = mpsc::channel(1_024);

// The command channel: The channel used to send requests from the session to the world
//let (command_tx, command_rx) = mpsc::unbounded_channel();
let (command_tx, command_rx) = mpsc::channel(1_024);


// Create the server
let sh = TelnetServer{
    clients: Arc::new(Mutex::new(HashMap::new())),
    id: 0,
    command_buffer: String::new(),
    tx_data_channel: Arc::new(Mutex::new(data_tx.clone())),
    tx_command_channel: Arc::new(Mutex::new(command_tx.clone()))
};

(sh, config, data_rx, command_rx)
}

/// Run a server.
/// Create a new `Connection` from the server's configuration, a
/// stream and a [`Handler`](trait.Handler.html).
pub async fn run<H: TelnetServer + Send + 'static>(
    config: Arc<Config>,
    addr: &str,
    mut server: H,
) -> Result<(), std::io::Error> {
    let addr = addr.to_socket_addrs().unwrap().next().unwrap();
    let socket = TcpListener::bind(&addr).await?;
    if config.maximum_packet_size > 65535 {
        error!(
            "Maximum packet size ({:?}) should not larger than a TCP packet (65535)",
            config.maximum_packet_size
        );
    }
    while let Ok((socket, _)) = socket.accept().await {
        let config = config.clone();
        let server = server.new(socket.peer_addr().ok());
        tokio::spawn(run_stream(config, socket, server));
    }
    Ok(())
}
