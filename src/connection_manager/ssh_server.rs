
use futures;
use tokio;
use std::pin::Pin;
use std::sync::Arc;
use thrussh::*;
use thrussh_keys::*;
use thrussh::server::{Auth, Session};
use tracing::{instrument, debug, error, info, warn};
use futures::FutureExt;
use anyhow;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use super::{Command, Data, DataMessage};
use termion::color;


#[derive(Clone, Debug)]
pub struct Server {
    client_id: usize,
    client_username: Option<String>,
    echo: bool,
    data_buffer: Data,
    tx_data_channel: Sender<DataMessage>,
    tx_command_channel: Sender<Command>, 
    server_allowed_keys: Vec<String>,
}

impl server::Server for Server {
    type Handler = Self;
    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.client_id += 1;
        s
    }
}

impl server::Handler for Server {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = Pin<Box<dyn futures::Future<Output = Result<(Self, Session), anyhow::Error>> + std::marker::Send>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth { futures::future::ready(Ok((self, auth))) }
    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool { futures::future::ready(Ok((self, s, b))) }
    fn finished(self, s: Session) -> Self::FutureUnit {
        Box::pin(futures::future::ready(Ok((self, s))))
    }

    fn auth_none(self, user: &str) -> Self::FutureAuth {
        info!("User {} tried to authenticate with method none. Denying.", user);
        futures::future::ready(Ok((self, server::Auth::Reject)))
    }

    #[instrument]
    fn auth_publickey(mut self, user: &str, pubkey: &key::PublicKey) -> Self::FutureAuth {
        // Thrussh will take care to verify that the client possesses the private
        // key. We only need to make sure that this is one of the allowed keys.
        //TODO - no verification yet implemented
        debug!("Server {}: Authenticating user {} with method public key.", self.client_id, user);
        debug!("Public Key is: {:?} with fingerprint {:?}", pubkey, pubkey.fingerprint());
        self.client_username = Some(user.to_string());
        for key in &self.server_allowed_keys {
            if key.eq_ignore_ascii_case(pubkey.public_key_base64().as_str()) {
                info!("Successfully authenticated {} by public key.", user);
                return futures::future::ready(Ok((self, server::Auth::Accept)));
            }
        }
        info!("Authentication by public key for {} failed: Identity not found.", user);
        futures::future::ready(Ok((self, server::Auth::Reject)))
    }

    #[instrument]
    fn auth_password(self, user: &str, password: &str) -> Self::FutureAuth {
        info!("User {} tried to authenticate with method password. Denying.", user);
        futures::future::ready(Ok((self, server::Auth::Reject)))
    }

    fn channel_open_session(self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        let handle = session.handle().clone();
        let registration_command = Command::Register(self.client_id, self.client_username.clone().unwrap(), channel, handle);
        async move {
            // Register client with the world - pass the handle to world thread
            //
            // This needs to be done to enable the world thread to send data to the
            // ssh user (eg. a description or a result).
            if let Err(_) = self.tx_command_channel.send(registration_command).await {
                error!("channel_open_session(): receiver dropped");
            } else {
                debug!("channel_open_session(): Sent client id and handle to world.")
            };

            // Display a welcome message
            session.data(channel,CryptoVec::from_slice(format!("{}Welcome.{}\r\n", color::Fg(color::Cyan), color::Fg(color::Reset)).as_ref()));
            Ok((self, session))
        }.boxed()
    }

    fn data(mut self, channel: ChannelId, data: &[u8], mut session: server::Session) -> Self::FutureUnit { 
        //Check if the data contains a CR, which is the indicator that the command
        //should either be processed by the ssh server or be sent to the world.
        let process_condition = data.as_ref() == "\u{000d}".as_bytes();
        let mut data_to_send = None;

        // If echo is on, then echo the received data back to the client
        // TODO - properly process deltion. Maybe add cursor movement and line editing.
        if self.echo {
            // We need to fix CR/LF as we only receive a CR when the user hits enter.
            // If we would not do this, then the next message sent to the client will
            // overwrite the echoed command (as the cursor is simply moved to the 
            // beginning of the line).
            if process_condition {
                session.data(channel, CryptoVec::from_slice("\r\n".as_ref()));
            } else {
                session.data(channel, CryptoVec::from_slice(data.clone()));
            }
        }
     
        // If CR was not hit, we append to the buffer. Otherwise we process the
        // buffer.
        if !process_condition {
            self.data_buffer.extend_from_slice(data);
        } else {
            // Evaluate if we deal with a command to the ssh server. If not,
            // send the data command to the world.
            // Currently there is only one server command implemented: Echo
            // TODO - implement hangup command
            if self.data_buffer.eq_ignore_ascii_case(b"echo on") {
                self.echo = true;
            } else if self.data_buffer.eq_ignore_ascii_case(b"echo off") {
                self.echo = false;
            } else if self.data_buffer.eq_ignore_ascii_case(b"echo") {
                self.echo = !self.echo;
            } else {
                // We have a data messge that we need to send to the world
                data_to_send = Some(self.data_buffer.clone());
            }
            // Data message was processed. Purge the buffer.
            self.data_buffer.clear();
        }

        let tx = self.tx_data_channel.clone();
        async move {
            match data_to_send {
                Some(data) => {
                    let data_message = DataMessage::new(self.client_id, data);
                    if let Err(_) = tx.send(data_message).await { 
                        println!("data(): receiver dropped");
                    };
                },
                None => {}
            }
            Ok((self, session))
        }.boxed()
    }

    fn signal(self, _channel: ChannelId, _signal_name: Sig, session: Session) -> Self::FutureUnit {
        warn!("Signal received but ignored.");
        Box::pin(futures::future::ready(Ok((self, session))))
    }
}

#[instrument]
pub fn init_ssh_server(allowed_keys: Vec<String>) -> (Server, Arc<thrussh::server::Config>,
                             Receiver<DataMessage>, Receiver<Command>) {
    // Configure the server
    let mut config = thrussh::server::Config::default();
    config.methods = MethodSet::PUBLICKEY | MethodSet::PASSWORD;
    config.connection_timeout = Some(std::time::Duration::from_secs(600));
    config.auth_rejection_time = std::time::Duration::from_secs(3);
    config.keys.push(thrussh_keys::key::KeyPair::generate_ed25519().unwrap());
    config.auth_banner = None;
    let config = Arc::new(config);

    // The data channel: The channel players use to send actions etc....
    let (data_tx, data_rx) = mpsc::channel(1_024);

    // The command channel: The channel used to send requests from the session to the world
    let (command_tx, command_rx) = mpsc::channel(1_024);


    // Create the server
    let sh = Server{
        client_username: None,
        client_id: 0,
        echo: false,
        data_buffer: Data::new(),
        tx_data_channel: data_tx.clone(),
        tx_command_channel: command_tx.clone(),
        server_allowed_keys: allowed_keys,
    };

    (sh, config, data_rx, command_rx)
}

#[derive(Debug, Clone)]
pub struct SSHKey {
    pub algorithm: String,
    pub key_base64: String,
    pub id: String,
}