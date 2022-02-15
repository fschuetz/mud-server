//! Module for Infrastructure Elements
//!
//! TODO.
pub mod ssh_server;
//pub mod telnet_server;

/// A type for client ids
pub type ClientId = usize;
/// A type for data
pub type Data = Vec<u8>;

/// Types for valid commands sent over the command channel from a connection
/// handler to the world.
#[derive(Clone)]
pub enum Command {
    /// Command to register new client and the communication channel to it
    Register(ClientId, String, thrussh::ChannelId, thrussh::server::Handle),
    /// Client request to terminate session
    Hangup(ClientId),
}

#[derive(Clone)]
pub struct DataMessage {
    pub client_id: ClientId,
    pub data: Data,
}


// unfortunately the standard library cannot provide
// a generic blanket impl to save us from this boilerplate
impl AsRef<DataMessage> for DataMessage {
    fn as_ref(&self) -> &DataMessage {
        self
    }
}


impl DataMessage {
    /// Generate a new data message
    /// 
    /// #Examples
    ///
    /// ```
    /// let message = DataMessage::new(0, Data::from("my data"));
    /// assert_eq!(message.client_id, 0);
    /// assert_eq!(message.data, "my data");
    /// ```
    pub fn new(client_id: ClientId, data: Data) -> DataMessage{
        DataMessage {
            client_id,
            data
        }
    }
}