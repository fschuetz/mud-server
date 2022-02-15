//! Properties
//! 
//! Contains properties that can be used for game assets (an other stuff if needed):
//!  * Node (the "room" that contain stuff)
//!  * Port (entry and exit points from nodes)
//!  * Connection (connections between ports that allow to travel from and to nodes)

use std::convert::TryFrom;
use crate::world::errors::Error;

/// Properties of game assets
#[derive(Debug)]
pub enum Property {
    Color(Color),
    Rigidity(Rigidity),
    Temperature(Temperature),
    Lighting(Lighting),

    // Wrapper for custom properties (avoid if possible)
    Custom(String),
}

/// Parse a string into a property
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// a property by trying to construct any of its members. The first member that
/// matches is constructed. If none can be created, then a custom property is
/// created.
/// 
/// TODO:
///     [] Currently does not support synonyms
impl From<&str> for Property {
    
    /// Try to parse a string into an action
    fn from(item: &str) -> Property {
        match Color::try_from(item) {
            Ok(c) => Property::Color(c),
            Err(_) => {
                match Rigidity::try_from(item) {
                    Ok(r) => Property::Rigidity(r),
                    Err(_) => {
                        match Temperature::try_from(item) {
                            Ok(t) => Property::Temperature(t),
                            Err(_) => {
                                match Lighting::try_from(item) {
                                    Ok(l) => Property::Lighting(l),
                                    Err(_) => Property::Custom(item.to_string()),
                                }
                            },
                        }
                    }
                }
            },
        }
    }
}

/// Color properties
#[derive(Debug)]
pub enum Color {
    Red,
    Blue,
    Green,
    Yellow,
    Cyan,
    Magenta,
    Black,
    White,
    Violet,
    Purple,
}

/// Try to parse a string into a color property
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// a color property.
/// 
/// TODO:
///     [] Currently does not support synonyms
impl TryFrom<&str> for Color {
    type Error = Error; 
    
    /// Try to parse a string into an action
    fn try_from(item: &str) -> Result<Self, Error> {
        match item.to_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "cyan" => Ok(Color::Cyan),
            "magenta" => Ok(Color::Magenta),
            "black" => Ok(Color::Black),
            "white" => Ok(Color::White),
            "violet" => Ok(Color::Violet),
            "purple" => Ok(Color::Purple),
            _ => return Err(Error::PropertyConversionFailed),
        }
    }
}

/// Rigidity properties
#[derive(Debug)]
pub enum Rigidity {
    Rigid,
    Solid,
    Liquid,
    Aerially,
    Frozen,
    Molten,
}

/// Try to parse a string into a rigidity property
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// a rigidity property.
/// 
/// TODO:
///     [] Currently does not support synonyms
impl TryFrom<&str> for Rigidity {
    type Error = Error; 
    
    /// Try to parse a string into an action
    fn try_from(item: &str) -> Result<Self, Error> {
        match item.to_lowercase().as_str() {
            "rigid" => Ok(Rigidity::Rigid),
            "solid" => Ok(Rigidity::Solid),
            "liquid" => Ok(Rigidity::Liquid),
            "aerially" => Ok(Rigidity::Aerially),
            "frozen" => Ok(Rigidity::Frozen),
            "molten" => Ok(Rigidity::Molten),
            _ => return Err(Error::PropertyConversionFailed),
        }
    }
}

/// Temperature properties
#[derive(Debug)]
pub enum Temperature {
    Cold,
    Cool,
    Warm,
    Hot,
}

/// Try to parse a string into a temperature property
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// a temperature property.
/// 
/// TODO:
///     [] Currently does not support synonyms
impl TryFrom<&str> for Temperature {
    type Error = Error; 
    
    /// Try to parse a string into an action
    fn try_from(item: &str) -> Result<Self, Error> {
        match item.to_lowercase().as_str() {
            "cold" => Ok(Temperature::Cold),
            "cool" => Ok(Temperature::Cool),
            "warm" => Ok(Temperature::Warm),
            "hot" => Ok(Temperature::Hot),
            _ => return Err(Error::PropertyConversionFailed),
        }
    }
}

/// Lighting properties
#[derive(Debug)]
pub enum Lighting {
    Pulsing,
    Radiating,
    Shining,
    Bright,
    Dark,
    Glowing,
}

/// Try to parse a string into a lighting property
/// 
/// This implementation of TryFrom attempts to deconstruct a given string into
/// a lgihting property.
/// 
/// TODO:
///     [] Currently does not support synonyms
impl TryFrom<&str> for Lighting {
    type Error = Error; 
    
    /// Try to parse a string into an action
    fn try_from(item: &str) -> Result<Self, Error> {
        match item.to_lowercase().as_str() {
            "pulsing" => Ok(Lighting::Pulsing),
            "radiating" => Ok(Lighting::Radiating),
            "shining" => Ok(Lighting::Shining),
            "bright" => Ok(Lighting::Bright),
            "dark" => Ok(Lighting::Dark),
            "glowing" => Ok(Lighting::Glowing),
            _ => return Err(Error::PropertyConversionFailed),
        }
    }
}