use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use iced::Column;
use iced_futures::Command;
use serde::{Serialize, Deserialize};

use crate::action::{Action, ID};
use crate::comm::{Message, Sender};
use crate::util::timestamp;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    #[serde(skip)]
    id: usize,
    #[serde(default)]
    title: String,
    #[serde(default, skip_serializing)]
    description: String,
    #[serde(default)]
    actions: Vec<Action>,
    #[serde(skip)]
    task_dir: String,
    #[serde(skip)]
    log_dir: String,
}

impl Block {
    pub fn init(&mut self, id: usize, task_dir: &Path) {
        self.id = id;
        if self.description.starts_with("~") {
            let file = task_dir.join(&self.description[1..]);
            let mut file = File::open(file)
                .expect("Failed to open block description file");

            self.description = String::new();
            file.read_to_string(&mut self.description)
                .expect("Failed to read block description file");
        }

        let mut next_id = 1;
        let mut last_action = None;
        for action in &mut self.actions {
            let (a, b) = action.init(next_id, &last_action, task_dir);
            next_id = a;
            last_action = b;
        }

        // todo!("Make sure deps and limits link to valid ids");
        for _action in &self.actions {
            //
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn actions(&self) -> Vec<ID> {
        self.actions
            .iter()
            .map(|x| x.id())
            .collect()
    }

    pub fn dependents(&self, id: &ID) -> HashSet<ID> {
        self.actions
            .iter()
            .filter(|x| {
                if let Some(other) = x.with() {
                    &other == id
                } else {
                    false
                }
            })
            .map(|x| x.id())
            .collect()
    }

    pub fn successors(&self, id: &ID) -> HashSet<ID> {
        self.actions
            .iter()
            .filter(|x| x.after().contains(id))
            .map(|x| x.id())
            .collect()
    }

    pub fn is_ready(&self, id: &ID, complete: &HashSet<ID>) -> Option<bool> {
        self.actions
            .iter()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .is_ready(complete)
    }

    pub fn has_view(&self, id: &ID) -> bool {
        self.actions
            .iter()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .has_view()
    }

    pub fn has_background(&self, id: &ID) -> bool {
        self.actions
            .iter()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .has_background()
    }

    pub fn captures_keystrokes(&self, id: &ID) -> bool {
        self.actions
            .iter()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .captures_keystrokes()
    }

    pub fn with_log_dir(mut self, log_dir: &str) -> Self {
        self.log_dir = Path::new(log_dir)
            .join(format!("block-{}-{}", self.id, timestamp()))
            .to_str().unwrap().to_string();
        std::fs::create_dir_all(&self.log_dir)
            .expect("Failed to create output directory for block");
        self
    }

    pub fn execute(&mut self, id: &str, writer: Sender) -> Command<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .run(writer, &self.log_dir)
    }

    pub fn update(&mut self, id: &ID, message: Message) -> Command<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .update(message)
    }

    pub fn view(&mut self, id: &ID) -> Column<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .view()
    }

    pub fn background(&mut self, id: &ID) -> Column<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .background()
    }

    pub fn wrap(&mut self, id: &ID) {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .wrap()
    }
}
