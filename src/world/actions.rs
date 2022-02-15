
//! Actions
//! 
//! Contains the different acitons that can be performed in the game.

use crate::world::properties::Property;
use std::fmt;

/// An enum denominating all the possible actions
pub enum Action {
    Look{target: Option<String>, preposition: Option<String>, properties: Option<Vec<Property>>}, //{target: Option<Box<dyn Observable + Send + Sync>>},
    Read,
    Enter,
    Connect,
    Access,
    Open,
}

/// Display an action
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Look { target, preposition, properties } => {
                // TODO - print the properties
                match target {
                    Some(t) => {
                        let  prep = match preposition {
                            Some(p) => format!("{} ", p),
                            None => "".to_string(),
                        };

                        let prop = match properties {
                            Some(_) => {
                                // TODO
                                "properties printing not yet implemented "
                            },
                            None => "",
                        };

                        write!(f, "look {}{}{}", prep, prop, t)
                    },
                    None => {
                        // There is no legitimate look command with prepostion and properties but no target
                        // so we need to only consider the one case.
                        write!(f, "look")
                    },
                }
            },
            Action::Read => write!(f, "read (todo)"),
            Action::Enter => write!(f, "enter (todo)"),
            Action::Connect => write!(f, "connect (todo)"),
            Action::Access => write!(f, "access (todo)"),
            Action::Open => write!(f, "open (todo)"),
        }
    }
}