use chrono::{DateTime, Utc};
use iced_native::keyboard::KeyCode;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;

use crate::aux::{rel_path, rel_path_from};
use crate::block::Block;
use crate::question::Summary;

pub fn timestamp(datetime: &DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%d %H-%M-%S%.3f UTC").to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Event<T: Block> {
    Init {
        task: String,
        version: String,
        sess_id: String,
        config: T::Config,
    },
    BlockStart {
        id: String,
    },
    BlockEnd {
        id: String,
        success: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reaction {
    block: String,
    time: String,
    key_code: String,
}

impl Reaction {
    pub fn new(block: String, key_code: KeyCode) -> Self {
        Reaction {
            block,
            time: timestamp(&Utc::now()),
            key_code: format!("{:?}", key_code),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    block: String,
    time: String,
    entry: Summary,
}

impl Response {
    pub fn new(block: String, entry: Summary) -> Self {
        Response {
            block,
            time: timestamp(&Utc::now()),
            entry,
        }
    }
}

#[derive(Debug)]
pub struct Logger<T: Block> {
    sid: String,
    uri: PathBuf,
    events: Vec<Event<T>>,
    reactions: Vec<Reaction>,
    responses: Vec<Response>,
}

impl<T: Block> Logger<T> {
    pub fn new() -> Logger<T> {
        let sid = timestamp(&Utc::now());
        let uri = rel_path(&format!("output/{}", sid));

        std::fs::create_dir_all(&uri).expect("Failed to create log directory.");

        Logger {
            sid,
            uri,
            events: vec![],
            reactions: vec![],
            responses: vec![],
        }
    }

    pub fn sess_id(&self) -> String {
        self.sid.clone()
    }

    pub fn log_event(&mut self, event: Event<T>) {
        self.events.push(event);

        let writer = File::create(rel_path_from(&self.uri, "event.txt"))
            .expect("Failed to create res.text file for logging events.");

        serde_json::to_writer_pretty(&writer, &self.events)
            .expect("Failed to write events to log file.");
    }

    pub fn log_reaction(&mut self, reaction: Reaction) {
        self.reactions.push(reaction);

        let writer = File::create(rel_path_from(&self.uri, "reaction.txt"))
            .expect("Failed to create res.text file for logging reactions.");

        serde_json::to_writer_pretty(&writer, &self.reactions)
            .expect("Failed to write reactions to log file.");
    }

    pub fn log_response(&mut self, response: Response) {
        self.responses.push(response);

        let writer = File::create(rel_path_from(&self.uri, "response.txt"))
            .expect("Failed to create res.text file for logging responses.");

        serde_json::to_writer_pretty(&writer, &self.responses)
            .expect("Failed to write responses to log file.");
    }
}
