//! A mud server for the balccon badge cyberpunk world
//! 
//! This file bootstraps the cyberpunk virtual world that can be accessed by using
//! the balccon badge as a cyberdeck.
#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

mod connection_manager;
mod world;
mod settings;
#[cfg(test)] mod tests;

#[macro_use] extern crate serde_derive;

use settings::Settings;
use tracing::{instrument, info, debug};
use world::GameWorld;
//use tracing_subscriber;
// use tracing_subscriber::EnvFilter;


#[instrument]
#[tokio::main]
async fn main() { 
    // Choose one of the subscribers, either console or tracing

    // EXPERIMENTAL - We use the experimental tokio-console to monitor threads
    // --> tokio-console can take a long time for recompiles, thus comment out
    //      if you experience problems during development and switch back on for
    //      testing.
    console_subscriber::init();
    
    // We use tracing for debug information as it is better suited for multitasking
    // applications than traditional logging. Set up the tracing subscriber here
    /*
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    */

    // Load the settings
    let settings = match Settings::new() {
        Ok(s) => {
            debug!("Settings loaded: {:?}", s);
            s
        },
        Err(e) => {
            panic!("Error reading settings: {}", e)
        },
    };

    // Extract allowed keys from config
    let mut allowed_keys = Vec::new();
    for key_info in settings.security.allowed_keys {
        allowed_keys.push(key_info[1].clone());
    }

    // Configure the ssh server
    let (sh, config,
        sender_data_rx, sender_command_rx)
        = connection_manager::ssh_server::init_ssh_server(allowed_keys);
    let mut addr = String::from(settings.ssh_server.host);
    addr.push_str(":");
    addr.push_str(settings.ssh_server.port.to_string().as_ref());

    // In this part we instantiate the world
    //
    // 1. Load the world configuration
    // 2. Run the world instance

    // TODO - Make world loadable from disk
    let mut world = GameWorld::new(format!("Testworld"));
    
    // Build first node and make it a spawn node
    // TODO - generate global array of assets
    let mut id_counter = 0;
    let mut node = world::assets::Node::new(id_counter);
    node.update_description("Around you its dark. You feel more than you see a \
        pulsing ultraviolet light.");
    
    id_counter += 1;
    let mut port = world::assets::Port::new(id_counter);
    port.update_description("A simple port that looks absolutely normal.");
    node.add_asset(Box::new(port));
    
    id_counter += 1;
    let mut port = world::assets::Port::new(id_counter);
    port.update_description("A port that has a slight purple shimmering edge.");
    node.add_asset(Box::new(port));
    world.add_spwan_node(node);

    //Increase ID counter for next node
    //id_counter += 1;


    // Spawn World Thread
    tokio::spawn(async move{
        world::run(sender_command_rx, sender_data_rx, world).await;
    });

    // Start the ssh server and listen for incoming connections
    //
    // Not that we do not need to spawn a thread but can just await the run function.
    // This is because the run function spawns a thread whenever a new client calls.
    // Otherwise it keeps looping and thus keeps our main function nice and active as
    // long as the server runs.
    info!("Spawning ssh server listening at: {}", addr);
    thrussh::server::run(config, addr.as_ref(), sh).await.unwrap();
}
