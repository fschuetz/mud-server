use std::result;
use std::io;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::env;

use tracing::debug;
use tracing::error;

/// Struct to describe the state machine of the BBS
/// Stores states in the form of nodes and transitions in the form of vectors
/// signifying conditions and the next state
pub enum ScreenType {
    Welcome,
}

impl ScreenType {

    pub fn display_ansi(&self) -> result::Result<Vec<u8>, io::Error> {
        match self {
            ScreenType::Welcome=> {
                // TODO set path in configuration an pass here
                let path: PathBuf = env::current_dir()
                    .unwrap()
                    .join("screens")
                    .join("00_welcome.ans");
                match File::open(path) {
                    Err(why) => {
                        error!("Couldn't open welcome screen: {}", why);
                        return Err(why)
                    },
                    Ok(file) => {
                        let mut buffered = io::BufReader::new(file);
                        let buf = &mut vec![];

                        match buffered.read_to_end(buf) {
                            Ok(_) => return Ok(buf.to_vec()),
                            Err(e) => return Err(e),
                        };
                    },
                };
            }
        }
    }
}

