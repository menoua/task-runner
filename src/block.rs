use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use iced::Column;
use iced_futures::Command;
use serde::{Serialize, Deserialize};

use crate::action::{Action, ID};
use crate::comm::{Message, Sender};
use crate::global::Global;
use crate::util::{timestamp, async_write_to_file};

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
    log_dir: String,
    #[serde(skip)]
    events: Vec<String>,
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

        let ids = HashSet::<ID>::from_iter(self.actions());
        for action in &self.actions {
            if !action.after().iter()
                .all(|id| ids.contains(id)) {
                panic!("IDs in `after` parameters should correspond to action IDs");
            }
            if let Some(id) = action.with() {
                if !ids.contains(&id) {
                    panic!("IDs in `with` parameters should correspond to action IDs");
                }
            }
            if action.after().contains(&action.id()) {
                panic!("An action cannot start after itself");
            }
            if action.with() == Some(action.id()) {
                panic!("An action cannot start with itself");
            }
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

    pub fn is_expired(&self, id: &ID, complete: &HashSet<ID>) -> Option<bool> {
        self.actions
            .iter()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .is_expired(complete)
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

    pub fn execute(&mut self, id: &ID, writer: Sender, global: &Global) -> Command<Message> {
        self.events
            .push(format!("{}  START  {}", timestamp(), id));

        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .run(writer, &self.log_dir, global)
    }

    pub fn update(&mut self, id: &ID, message: Message, global: &Global) -> Command<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .update(message, global)
    }

    pub fn view(&mut self, id: &ID, global: &Global) -> Column<Message> {
        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .view(global)
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
        self.events
            .push(format!("{}  WRAP  {}", timestamp(), id));

        self.actions
            .iter_mut()
            .filter(|x| x.is(id))
            .next()
            .unwrap()
            .wrap()
    }

    pub fn skip(&mut self, id: &ID) {
        self.events
            .push(format!("{}  SKIP  {}", timestamp(), id));
    }

    pub fn finish(&mut self) {
        async_write_to_file(
            Path::new(&self.log_dir).join("events.log").to_str().unwrap().to_string(),
            self.events.clone(),
            "Failed to write block event log to output file");
        // let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
        // serde_yaml::to_writer(file, &self.events)
        //     .expect("Failed to write block event log to output file");
        self.events.clear();
    }
}
