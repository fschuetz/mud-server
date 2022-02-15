//! Grammar
//! 
//! Defines the grammar that can be used in the game world and how this grammar
//! is mapped to data structures for use in the game.
//! 
//! The grammar supported is:
//! ```ignore
//!     <sentence> ::= <action> | <command>
//!     <action> ::= <verb> <blank> <adverblist> <blank> <object> ("." | E)
//!     <command> ::= "help" (<blank> <topic> | E) | "inventory"
//!     <adverblist> ::= <adverb> | <adverb> (","+ <blank>* | <blank>+) <adverblist> | E
//!     <adverb> ::= "quickly" | "slowly"
//!     <do> ::= "do"
//!     <verb> ::= "look" | "read" | "enter" | "connect" | "access" | "open"
//!     <object> ::= <article> ("port" | "ram bank" | "quickhack")
//!     <article> ::= ("the" <blank> | E)
//!     <topic> ::= "verbs" | "inventory" | "combat" 
//!     <blank> ::= " "+
//! ```
//! 
//! TODO:
//! - [ ] Maybe use lexxer / parser
//! - [ ] Define sentence structures
//! - [ ] Clean up traits identifiable, observable, interactable or should we
//!         use a generic interacable trait that then reacts upon the action enum?
//! - [ ] Ensure grammar is up to date

use std::convert::TryFrom;
use tracing::{debug, info, error};

use crate::world::errors::Error;
use super::actions::Action;

use regex::Regex;
use lazy_static::lazy_static;

use crate::world::properties::Property;


/// Try to parse a string into an action
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// an action type.
/// 
/// TODO:
///     [] Currently only supports single word, make full sentence parser
impl TryFrom<&str> for Action {
    type Error = Error; 
    
    /// Try to parse a string into an action
    fn try_from(item: &str) -> Result<Self, Error> {
        // Get the first word (until either newline or whitespace)      
        lazy_static! {
            static ref CMD_RE: Regex = Regex::new(r"^([\w\-]+)").unwrap();
        }
        let mat = CMD_RE.find(item).unwrap();
        let command = &item[mat.start()..mat.end()];

        // Check if the first word is a legitimate command and then depending
        // on the command desstructure further.
        for i in synonyms(command) {
            match i.to_lowercase().as_str() {
                "look" => {
                    if mat.end() == item.len() {
                        // No more remaining characters. We have a simple "look" command.
                        debug!("Found simple look command: \"{}\"", command);
                        return Ok(Action::Look {target: None, preposition: None, properties: None});
                    } else {
                        debug!("Found command \"{}\". Rest of data message is \"{}\"", command, &item[mat.end()+1..]);
                    
                        // Try to match either a simple look command or a complex look command
                        // For a simple look command only whitespaces and an optional dot may follow.
                        lazy_static! {
                            static ref LOOK_RE: Regex = Regex::new(r"^\s*\.?\s*$").unwrap();
                        }
                        let look_command = LOOK_RE.find(&item[mat.end()..]);

                        match look_command {
                            Some(m) => {
                                // There are only whitespaces and an optional dot. 
                                // It is a simple look command. Return without target.
                                return Ok(Action::Look 
                                    {
                                        target: None, 
                                        preposition: None, 
                                        properties: None
                                    }
                                );
                            },
                            None => {
                                // For a complex look command we need an adverb, 
                                // zero or more adjectives and a noun.
                                // TODO - maybe we could extract adjectives in 
                                // one run by adjusting first reges
                                lazy_static! {
                                    static ref COMPLEX_LOOK_RE: Regex 
                                        = Regex::new(r"^\s*\b(\p{L}+)\s+((?:\b(?:\p{L}+)\b(?:\s*,\s*|\s+))*)\b(\p{L}+)\s*\.?\s*$").unwrap();
                                }
                                let cap = COMPLEX_LOOK_RE.captures(&item[mat.end()..]);
                                //match COMPLEX_LOOK_RE.find(&item[mat.end()..]) {
                                match cap {
                                    Some(caps) => {
                                        info!("Complex command found: {:?}", caps);
                                        // Our capture must match 4 groups (the full match and the groupd)
                                        // Otherwise something went wrong
                                        if caps.len() != 4 {
                                            error!("Invalid complex \"look\" command structure ok.");
                                            return Err(Error::VerbEncodingError);
                                        }
                                        
                                        // Extract all the properties.
                                        let properties = caps.get(2).map_or(None, |m| {
                                            let mut p = Vec::new();

                                            lazy_static! {
                                                static ref PROP_RE: Regex = Regex::new(r"([\s*\p{L}]+?)(?:\s*,\s*|\s+|$)").unwrap();
                                            }
                                            // TODO map string on properties
                                            // TODO error handling
                                            for cap in PROP_RE.captures_iter(m.as_str()) {
                                                let property_str = cap.get(1).map_or("", |m| m.as_str());

                                                // Try to build a property
                                                p.push(Property::from(property_str));
                                            }
                                            Some(p)
                                        });
                                
                                        // TODO set properties
                                        return Ok(Action::Look {
                                            target: caps.get(3).map_or(None, |m| Some(m.as_str().to_string())), 
                                            preposition: caps.get(1).map_or(None, |m| Some(m.as_str().to_string())), 
                                            properties
                                        });
                                    },
                                    None => {
                                        info!("No complex command found.");
                                    },
                                }
                            },
                        }
                    }
                },
                "read" => return Ok(Action::Read),
                "enter" => return Ok(Action::Enter),
                "connect" => return Ok(Action::Connect),
                "Access" => return Ok(Action::Access),
                "Open" => return Ok(Action::Open),
                _ => {},
            }
        };

        Err(Error::VerbUnknownError)
    }
}

/// Try to parse a Vec<u8> into an action
/// 
/// This implementation of TryFrom attempts to deconstruct a given vector of u8
/// into an action. It does so by first trying to construct a str from the 
/// bytes in the vector and then calls the uses the TryFrom implementation for
/// str to do the deconstruction.
impl TryFrom<Vec<u8>> for Action {
    type Error = Error; 
    fn try_from(item: Vec<u8>) -> Result<Self, Error> {

        // Decode to &str
        let s = match std::str::from_utf8(&item) {
            Ok(v) => v,
            Err(e) => {
                debug!("Could not generate verb from Vec<u8>: {}", e);
                return Err(Error::VerbEncodingError)
            },
        };

        Action::try_from(s)
    }
}

/// Helper function to give a list of synonymous words. Returns a vector only
/// containing the looked up word itself if no synonyms are available (every
/// word is synonymous to istself) and a vector of more sysnonyms otherwise also
/// including the word itself.
/// 
/// TODO:
/// - [ ] Implement it - currently just returns the word itself.
fn synonyms(word: &str) -> Vec<&str> {
    let mut synonyms = Vec::new();
    synonyms.push(word);
    synonyms
}