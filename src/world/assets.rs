//! Game assets
//! 
//! Contains all game assets:
//!  * Node (the "room" that contain stuff)
//!  * Port (entry and exit points from nodes)
//!  * Connection (connections between ports that allow to travel from and to nodes)

use super::actions::Action;
use super::properties::Property;

// TODO start using generational indices
pub type AssetID = u64;

/// Trait that is common to all game assets
pub trait GameAsset : std::fmt::Debug + Send + Sync {
    /// UID
    /// 
    /// Get the unique id of the asset
    fn uid(&self) -> AssetID;

    /// Name
    /// 
    /// Get the name of the asset
    fn name(&self) -> String;

    /// Properties
    /// 
    /// Return the properties of the asset
    fn properties(&self) -> Option<&Vec<Property>>;

    /// Describe
    /// 
    /// Describes the game asset. Depending on the asset type 
    /// contained assets may be added to the description to make
    /// it complete.
    /// 
    /// TODO - maybe remove as redundant due to interact
    fn describe(&self) -> String;

    /// React to
    /// 
    /// React to an interaction with the game asset. Interaction are based on
    /// verbs. The object responses to the verb by returning an Option, NONE
    /// if the object canot reatct to this and Some() if it does.
    /// 
    /// TODO - maybe add the subject that does the interaction to the signature
    /// TOTO - return a more generic result than String
    fn react_to(&self, a: &Action) -> String;
}

/// Structure that descibes a node
#[derive(Debug)]
pub struct Node {
    uid: AssetID,
    name: String,
    properties: Option<Vec<Property>>,
    description: String,
    sub_assets: Vec<Box<dyn GameAsset>>,
}

impl Node {
    /// Create a new empty node
    pub fn new(uid: AssetID) -> Node {
        let name = String::from("");
        let properties = None;
        let description = String::from("");
        let sub_assets = Vec::new();
        Node { uid, name, properties, description, sub_assets }
    }

    /// Update the description of the node
    pub fn update_description(&mut self, description: &str) {
        self.description = String::from(description);
    }

    /// Add a port to this node. If the node already has this port nothing
    /// is added.
    pub fn add_asset(&mut self, asset: Box<dyn GameAsset>) {
        // TODO - decide on how to deal with doublettes
        if !self.sub_assets.iter().any(|a| a.uid() == asset.uid()) {
            self.sub_assets.push(asset);
        }
    }

    /// Remove a port from this node. If a port is multiple times in the node,
    /// then all occurences will be removed (as this should never be the case).
    pub fn remove_asset(&mut self, asset_uid: AssetID) {
        self.sub_assets.retain(|a| a.uid() == asset_uid);
    }
}

impl GameAsset for Node {
    /// Returns the uid of the node
    fn uid(&self) -> AssetID {
        self.uid
    }

    /// Returns the node number
    /// 
    /// Node numbers are usually not known by default, but once discovered
    /// they may be used to manipulate nodes or fast travel.
    fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the properties of the node
    /// 
    /// TODO - maybe use some node properties to induce eg. damage to player
    fn properties(&self) -> Option<&Vec<Property>> {
        match &self.properties {
            Some(p) => Some(&p),
            None => None,
        }
    }

    /// Describes the room an all visible objects in it
    /// 
    /// TODO - deal with empty descriptions
    fn describe(&self) -> String {
        self.description.clone()
    }

    /// React to
    /// 
    /// Response to interactions with this node depending on the verb
    fn react_to(&self, a: &Action) -> String {
        match a {
            Action::Look{ target: None, ..} => {
                let mut description = format!("{}\r\n", self.description.clone());
                for asset in self.sub_assets.iter() {
                    description += format!("{}\r\n", asset.describe()).as_str();
                }
                description
            },
            Action::Look{ target: Some(t), preposition, properties} => {
                // TODO
                let description = format!("Not implemented!\r\n");
                description
            }
            Action::Read => format!("Read what?"),
            Action::Enter => format!("Enter what?"),
            Action::Connect => format!("Connect to what?"),
            Action::Access => format!("Access what?"),
            Action::Open => format!("Open what?"),
        }
    }
}

/// Port
/// 
/// A port is used to move from one node to others. A port can be connected to
///     * no other nodes (NONE)
///     * one other node
///     * multiple other nodes
/// TODO - we need to somehow implement how to choose the destination node if 
///         a port leads to multiple other nodes.
/// Ports can either be open - thus accessible - or closed - and thus inaccessible.
/// A closed port can be protected by ICE or other means. In that case in order to use
/// the port the barrier must first be removed. 
/// TODO - decide if we need to add visibility flag or if we update description for visibility changes
#[derive(Debug)]
pub struct Port {
    id: AssetID,
    properties: Option<Vec<Property>>,
    is_open: bool,
    connects_to: Option<Vec<Node>>,
    description: String,
    // TODO: Protections etc.....
}

impl Port {
    /// Create a new port
    pub fn new(id: AssetID) -> Port {
        Port {
            id,
            properties: None,
            is_open: false,
            connects_to: None,
            description: format!(""),
        }
    }

    /// Get the id 
    /// TODO - remove
    pub fn get_id(&self) -> AssetID { self.id }

    /// Describe a port
    pub fn update_description(&mut self, description: &str) {
        self.description = String::from(description);
    }
}

impl GameAsset for Port {
    /// Return the uid of the port
    fn uid(&self) -> AssetID {
        self.id
    }
    
    /// Returns the port id
    /// 
    /// TODO - maybe replace with something else?
    fn name(&self) -> String {
        "port".to_string()
    }

    /// Returns the properties of the node
    /// 
    /// TODO - maybe use some node properties to induce eg. damage to player
    fn properties(&self) -> Option<&Vec<Property>> {
        match &self.properties {
            Some(p) => Some(&p),
            None => None,
        }
    }

    /// Describe the port
    fn describe(&self) -> String {
        //TODO
        if self.is_open {
            format!("{} The port is open.", self.description)
        } else {
            format!("{} The port is closed.", self.description)
        }
    }

    /// React to
    /// 
    /// Response to interactions with this node depending on the verb
    fn react_to(&self, a: &Action) -> String {
        match a {
            Action::Look { target: None, .. } => {
                if self.is_open {
                    format!("{}\n The port is open.", self.description)
                } else {
                    format!("{}\n The port is closed.", self.description)
                }
            },
            Action::Look{ target: Some(_t), preposition, properties} => {
                // TODO -- try to find out what child object the interacting thing wants to
                // look at.
                let description = format!("Not implemented!\r\n");
                description
            }
            Action::Read => format!("Read what?"),
            Action::Enter => format!("Enter what?"),
            Action::Connect => format!("Connect to what?"),
            Action::Access => format!("Access what?"),
            Action::Open => format!("Open what?"),
        }
    }
}