//! Game world module
//!
//! This module encapsulates all the information about the game wold and the 
//! objects (including players) and interactions in it.
pub mod states;
pub mod assets;
pub mod grammar;
pub mod errors;
pub mod properties;
pub mod actions;

use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use crate::{connection_manager::{Command, DataMessage, ClientId}, world::states::ScreenType};

use thrussh::CryptoVec;
use tracing::{info, error, instrument, debug, warn};

use assets::GameAsset;
use actions::Action;
use std::convert::TryFrom;

use generational_arena::{Arena, Index};

/// Run
/// 
/// Run the world and accept commands from the connection manager for users to manipulate
/// the world.
#[instrument]
pub async fn run(mut command_rx: Receiver<Command>, mut data_rx: Receiver<DataMessage>, world: GameWorld) {
    
    let mut players : HashMap<ClientId, Player>= HashMap::new();
    loop {
        tokio::select! {
            // A game command was received. Process the command.
            Some(command) = command_rx.recv() => {
                debug!("Received command. Processing... (BLOCKING)");
                process_command(command, &world, &mut players).await;
            }

            // A player performed an interaction with the game world (data command). Process it.
            Some(data_message) = data_rx.recv() => {
                debug!("Received data. Processing: {:?} from data_tx of client {}", data_message.data, data_message.client_id);
                process_data(data_message, &world, &players).await;   
            }
            else => {
                error!("Both channels closed");
            }
        }
    } 
}

/// Handle commands
/// 
/// This function processes commands to the game engine. Commands are usually
/// issued by a client.
async fn process_command(command: Command, world: &GameWorld, players : &mut HashMap<ClientId, Player>) {
    match command {
        // Register a new player to the game
        Command::Register(client_id, username, channel_id, mut handle) => {
            // TODO - check if player is alread registered and using another session
            let mut player = Player::new(username, (channel_id, handle.clone()));
            match world.spawn(&mut player) {
                Ok(_) => {
                    players.insert(client_id, player);

                    // Display the welcome screen
                    // Open the file for the welcome screen and display it. If the file is not found
                    // (an error is sent to stderr and nothing is sent back to the client.)
                    match ScreenType::Welcome.display_ansi() {
                        // If we receive a valid screen, we send it on the channel. Otherwise we send nothing
                        // and write an error message to stderr
                        Ok(buf) => {
                            //session.data(channel, None, buf.as_ref());
                            handle.data(channel_id, CryptoVec::from_slice(
                                buf.as_ref()))
                                .await.expect("Could not send registration msg.");
                        },
                        Err(e) => error!("Error sending welcome screen to client: {}", e),
                    };
                },
                Err(_) => todo!(), // TODO - Send error screen and kill the conneciton
            };
        },
        Command::Hangup(_) => todo!(),
    };
}

/// Handle data messages
/// 
/// A data message usually is a player action. This function tries to decode
/// the data message and then act accordingly.
async fn process_data(data_message: DataMessage, world: &GameWorld, players: &HashMap<ClientId, Player>) {
    // Check if the data message can be matched on an active player. If no
    // active player is known then the data message gets discarded.
    match players.get(&data_message.client_id) {
        Some(player_info) => {

            // Check if the player did a proper action
            match Action::try_from(data_message.data.clone()) {
                Ok(a) => {
                    info!("Player {} is performing action {}.", player_info.player_name, a);

                    // Currently all our actions are location specific, so get the location of the player
                    match player_info.location {
                        Some(l) => {
                            // Currently all locations are nodes. So we only need to check if the node exists.
                            // If the node does not exist, we have some inconsistency.
                            match world.nodes.get(l) {
                                Some(node) => {
                                    // Send the action to the node. The node itself will take care to
                                    // relay the action to the necessary contents of itself.
                                    //
                                    // TODO - this mechanism currently limits action radius to one node
                                    //          we may want to implement either other nodes receiveing as well
                                    //          or even a generic listener that sends it to all assets?
                                    let response_message = node.react_to(&a);

                                    player_info.active_session.1.clone().data(player_info.active_session.0, 
                                        CryptoVec::from_slice(format!("{}\r\n",response_message).as_ref()))
                                        .await.expect("Could not send data message to client.".as_ref());
                                },
                                None => {
                                    error!("Location index cannot be mapped to node: {:?}", l);
                                    player_info.active_session.1.clone().data(player_info.active_session.0, 
                                        CryptoVec::from_slice("A glitch in the matrix occured.\r\n".as_ref()))
                                        .await.expect("Could not send data message to client.".as_ref());
                                },
                            }
                        },
                        None => {
                            // Check if this action is location independent - TODO currently no actions are location independen
                            warn!("User does not have a location. Command ignored.");
                            let message = "In limbo everything is possible. And nothing. Makes you wonder...\r\n";
                            player_info.active_session.1.clone().data(player_info.active_session.0, 
                                CryptoVec::from_slice(message.as_ref()))
                                .await.expect("Could not send data message to client.".as_ref());
                        },
                    }

                },
                Err(e) => {
                    // Not a valid aciton, tell the player
                    debug!("User used unkown command: {}", e);
                    let message = "Error 23: Command not found.\r\n";
                            player_info.active_session.1.clone().data(player_info.active_session.0, 
                                CryptoVec::from_slice(message.as_ref()))
                                .await.expect("Could not send data message to client.".as_ref());

                },
            }
        },
        None => error!("Received data message but no active player found for the client that sent the message."),
    };
}

/// GameWorld
/// 
/// The structure describing the game world.
#[derive(Debug)]
pub struct GameWorld {
    name: String,
    description: Option<String>,
    spawn_nodes: Vec<Index>, 
    nodes: Arena<assets::Node>,
    players: Vec<Player>, // Not sure we should include the players in the world? TODO replace with arena
}

impl GameWorld {
    /// Create a new GameWorld
    pub fn new(name: String) -> Self {
        GameWorld {
            name,
            description: None,
            spawn_nodes: Vec::new(),
            nodes: Arena::new(),
            players: Vec::new(),
        }
    }

    /// Add a node to the game world and marks it as a spawn node
    /// 
    /// If the world did not have this node present, None is returned.
    /// If the world did have this node present, the node is updated, and the old node is returned. 
    /// TODO - how to add something that tells us how to choose the node
    /// TODO - ensure update of node if node iwth $id exists.
    pub fn add_spwan_node(&mut self, node: assets::Node) -> Option<Index> {
        let idx = self.nodes.insert(node);
        self.spawn_nodes.push(idx);
        Some(idx)
    }

    /// Add a node to the game world
    /// 
    /// If the world did not have this node present, None is returned.
    /// If the world did have this node present, the node is updated, and the old node is returned. 
    /// TODO - how to add something that tells us how to choose the node
    /// TODO - ensure update of node if node iwth $id exists.
    pub fn add_node(&mut self, node: assets::Node) -> Option<Index> {
        // TODO - iterate over arena to check if the node with ID is already in the arena
        Some(self.nodes.insert(node))
    }

    /// Automatically choose a spawn node
    /// 
    /// Automatically chooses a spawn node for the given asset.
    /// TODO - Fix seleciton algorithm, currently we just take the first.
    /// TODO - Create specific spwan errors
    pub fn spawn<T>(&self, asset: &mut T) -> Result<Index, errors::Error> 
        where T: Spawnable {
        
        // TODO - choose better spawn point.

        if self.spawn_nodes.is_empty() {
            return Err(errors::Error::NoSpawnpointFound);
        } else {
            asset.set_spawn_point_index(self.spawn_nodes[0]);
            return Ok(self.spawn_nodes[0]);
        }
        
    }
}

/// A trait for spawnable objects
/// 
/// An object that can be spawned in different locations needs to implement
/// spwanable.
pub trait Spawnable {
    /// Add the object at index as a potential spawn point
    fn set_spawn_point_index(&mut self, index: Index);
}

/// A trait for assets that can be identified and referenced by other objects
/// 
/// In order to reference an asset one must be able to describe it in a way 
/// that unambiguously identifies the asset. Whether an identification is 
/// unambigous or not is depending on the context. For example, if there is a
/// node with only one "shiny, red port", then this port can be uniquely 
/// referenced either by "red port", "shiny port", "shiny, red port" or "port".
/// If there is a node that has more than one port, for example a "shiny, red
/// port" and a "shiny, blue port", then neither "shiny port" nor "port" 
/// uniquely identifies them. However, "red port", "shiny, red port", "blue
/// port" and "shiny, blue port" gives a unique identificaiton.
/// 
/// In our logic, we leave it to the game engine to figure out when an object
/// is uniquely identified (or even if this is necessary). On our assets that
/// must be identifiably (for example to allow interaciton) we simply require
/// that they can reply to the quesiton "does this attribute match you". So for
/// example, our "shiny, red port" would match "shiny", "red" and "port" or any
/// combination thereof. (Note that we could of course also only react to "red"
/// and "port" as a design choice to remove irrelevant attributes)
pub trait Identifiable {
    /// Returns true if the object can be identified by a given property
    fn has_property() -> bool;

    /// Returns true if the asset can be identified as an object
    fn is_object() -> bool;
}

/// A trait for assets that can be observed
/// 
/// If observed, the object that is under observation will reply with an
/// aciton. This can be a simple action such as for example juest gibing
/// a better description of the object, or it can be a complex action (eg.
/// if the object under observation is a person it can flee).
pub trait Observable {
    /// Returns true if the object can be identified by a given property
    fn observe(&self) -> Action;
}

struct Player {
    player_name: String,
    active_session: (thrussh::ChannelId, thrussh::server::Handle),
    location: Option<Index>,
}

impl Player {
    pub fn new(player_name: String, active_session: (thrussh::ChannelId, thrussh::server::Handle)) -> Player {
        Player {
            player_name,
            active_session,
            location: None,
        }
    }
}

impl Spawnable for Player {
    fn set_spawn_point_index(&mut self, index: Index) {
        self.location = Some(index);
    }
}

// TODO: We should somehow give information about the session
impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Player")
         .field("player_name", &self.player_name)
         .field("player_location", &self.location)
         .finish()
    }
}