/// Integration tests for the mud-server
///
use std::sync::Arc;
use thrussh::*;
use thrussh::server::{Auth, Session};
use thrussh::server::Handler;
use thrussh_keys::*;
use thrussh_keys::key::KeyPair;
use futures::Future;
use std::io::Read;
use crate::settings::Settings;
use crate::connection_manager;
use crate::connection_manager::ssh_server::Server;

/// Verify pbulic key as allowed
///
/// Test must successfully accept the public key as a accepted key.
/// (Signature is not verified, user not authenticated)
#[tokio::test]
async fn check_accepted_pubkey_ssh_ed25519() {
    // Setup the test environment
    let mut allowed_keys = Vec::new();
    let key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    allowed_keys.push(key);
    let test_environment =
        TestEnvironment::setup(allowed_keys);

    let result = test_environment.server
        .auth_publickey("testuser", &test_environment.keys[0].clone_public_key());
    let (_, auth_result) = result.into_inner().unwrap(); 
    assert_eq!(auth_result, thrussh::server::Auth::Accept);
}

/// Verify public key as not allowed
///
/// Test must not accept the public key as an accepted key.
#[tokio::test]
async fn check_rejected_pubkey_ssh_ed25519() {
    // Setup the test environment
    let mut allowed_keys = Vec::new();
    let key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    allowed_keys.push(key);
    let test_environment =
        TestEnvironment::setup(allowed_keys);

    let rejected_key =  thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
    let result = test_environment.server
        .auth_publickey("testuser", &rejected_key.clone_public_key());
        
    let (_, auth_result) = result.into_inner().unwrap(); 
    assert_eq!(auth_result, thrussh::server::Auth::Reject);
}

/** A structure that serves the test environment

    This structure cares for loading the settings and potential other 
    housekeeping tasks that are the same for multiple tests.

    TODO: We need to find a way to spawn a server on a free port without 
            interfering with other test environments. Maybe we can store
            the used ports in a shared variable or the like.
**/
pub struct TestEnvironment {
    settings: Settings,
    keys: Vec<thrussh_keys::key::KeyPair>,
    client_config: Arc<thrussh::client::Config>,
    server_config: Arc<thrussh::server::Config>,
    client: Client,
    server: Server,
}

impl TestEnvironment {
    fn setup(keys: Vec<KeyPair>) -> TestEnvironment {
        // Load settings from configuration files or environment variable
        // Values of variables overwrite the value for the same variable from
        // Settings.toml which overwrites DefaultSettings.toml if present.
        let settings = match Settings::new() {
            Ok(s) => s,
            Err(e) =>  panic!("Error reading settings: {}
            Aborting...",e),
        };

        // Generate an ssh client configuration
        let client_config = thrussh::client::Config::default();
        let client_config = Arc::new(client_config);
        let client = Client{};

        // TODO spawn a server (configured to accept keys) - this will be 
        //      difficult as tests run in parallel and we need to find a 
        //      way to bind to an unused port.
        // Configure the ssh server
        let mut allowed_keys : Vec<String> = Vec::new();
        for key in &keys {
            allowed_keys.push(key.public_key_base64());
        }
        let (server, server_config,
            sender_data_rx, sender_command_rx)
            = connection_manager::ssh_server::init_ssh_server(allowed_keys);
        let mut addr = String::from(settings.ssh_server.host.clone());
        addr.push_str(":");
        addr.push_str(settings.ssh_server.port.to_string().as_ref());

        TestEnvironment {
            settings,
            keys,
            client_config,
            server_config,
            client,
            server,
        }
    }
}

struct Client {
}

impl client::Handler for Client {
   type Error = anyhow::Error;
   type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;
   type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;

   fn finished_bool(self, b: bool) -> Self::FutureBool {
       futures::future::ready(Ok((self, b)))
   }
   fn finished(self, session: client::Session) -> Self::FutureUnit {
       futures::future::ready(Ok((self, session)))
   }
   fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
       println!("check_server_key: {:?}", server_public_key);
       self.finished_bool(true)
   }
   fn channel_open_confirmation(self, channel: ChannelId, max_packet_size: u32, window_size: u32, session: client::Session) -> Self::FutureUnit {
       println!("channel_open_confirmation: {:?}", channel);
       self.finished(session)
   }
   fn data(self, channel: ChannelId, data: &[u8], session: client::Session) -> Self::FutureUnit {
       println!("data on channel {:?}: {:?}", channel, std::str::from_utf8(data));
       self.finished(session)
   }
}

/* 
impl Drop for TestEnvironment {

    fn drop(&mut self) {
        // Here we should drop connections and do cleanup.
    }
}
*/

/*
    Miscellaneous helper functions
 */

