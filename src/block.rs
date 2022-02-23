use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use iced::Column;
use iced_futures::Command;
use serde::{Serialize, Deserialize};

use crate::action::{Action, flow, ID};
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
    id2action: HashMap<ID, usize>,
    #[serde(skip)]
    log_dir: String,
    #[serde(skip)]
    events: Vec<String>,
}

impl Block {
    pub fn init(&mut self, id: usize, task_dir: &Path) -> Result<(), String> {
        self.id = id;
        if self.description.starts_with("<") {
            let file = task_dir.join(&self.description[1..].trim());
            let mut file = File::open(file)
                .or(Err("Failed to open block description file".to_string()))?;

            self.description = String::new();
            file.read_to_string(&mut self.description)
                .or(Err("Failed to read block description file".to_string()))?;
        }

        let mut last_action = None;
        let mut ids = HashSet::new();
        for (i, action) in self.actions.iter_mut().enumerate() {
            action.init(i+1, &last_action, 0, task_dir)?;
            last_action = Some(action.id());

            let id = action.id();
            if ids.contains(&id) {
                return Err(format!("Action ID `{}` used more than once; IDs should be unique", id));
            } else {
                ids.insert(id);
            }
        }

        let mut i: usize = 0;
        while i < self.actions.len() {
            if matches!(self.actions[i], Action::Template { .. }) {
                if let Action::Template { actions: inners, .. } = self.actions[i].clone() {
                    self.actions.remove(i);
                    for inner in inners.into_iter() {
                        self.actions.insert(i, inner);
                        i += 1;
                    }
                }
            } else {
                i += 1;
            }
        }

        flow::add_gates(&mut self.actions, Some(HashSet::new()), None)?;

        // Make a lookup table for actions by ID
        for (i, action) in self.actions.iter().enumerate() {
            self.id2action.insert(action.id(), i);
        }
        let id_list: HashSet<ID> = self.id2action.keys().cloned().collect();

        // Verify basic action dependency logic
        for action in &mut self.actions {
            action.verify(&id_list)?;
        }

        // Make reverse dependency links
        for id in id_list {
            let action = self.action(&id)?;
            let (link_id, after, with) = (
                action.id(), action.after(), action.with());
            for id in after {
                self.action_mut(&id)?.add_successor(link_id.clone());
            }
            if let Some(id) = with {
                self.action_mut(&id)?.add_dependent(link_id);
            }
        }

        Ok(())
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
            .map(Action::id)
            .collect()
    }

    pub fn action(&self, id: &ID) -> Result<&Action, String> {
        let index = self.id2action.get(id)
            .ok_or(format!("Invalid reference to action: {}", id))?;
        Ok(&self.actions[*index])
    }

    pub fn action_mut(&mut self, id: &ID) -> Result<&mut Action, String> {
        let index = self.id2action.get(id)
            .ok_or(format!("Invalid mutable reference to action: {}", id))?;
        Ok(&mut self.actions[*index])
    }

    pub fn dependents(&self, id: &ID) -> &HashSet<ID> {
        &self.action(id).unwrap().dependents()
    }

    pub fn successors(&self, id: &ID) -> &HashSet<ID> {
        &self.action(id).unwrap().successors()
    }

    pub fn is_ready(&self, id: &ID) -> Option<bool> {
        self.action(id).unwrap().is_ready()
    }

    pub fn is_expired(&self, id: &ID) -> Option<bool> {
        self.action(id).unwrap().is_expired()
    }

    pub fn has_view(&self, id: &ID) -> bool {
        self.action(id).unwrap().has_view()
    }

    pub fn has_background(&self, id: &ID) -> bool {
        self.action(id).unwrap().has_background()
    }

    pub fn captures_keystrokes(&self, id: &ID) -> bool {
        self.action(id).unwrap().captures_keystrokes()
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
        let log_dir = self.log_dir.to_owned();
        self.events.push(format!("{}  START  {}", timestamp(), id));
        self.action_mut(id).unwrap().run(writer, &log_dir, global)
    }

    pub fn update(&mut self, id: &ID, message: Message, global: &Global) -> Command<Message> {
        self.action_mut(id).unwrap().update(message, global)
    }

    pub fn view(&mut self, id: &ID, global: &Global) -> Column<Message> {
        self.action_mut(id).unwrap().view(global)
    }

    pub fn background(&mut self, id: &ID) -> Column<Message> {
        self.action_mut(id).unwrap().background()
    }

    pub fn satisfy(&mut self, id: &ID) -> (HashSet<ID>, HashSet<ID>) {
        let mut ready = HashSet::new();
        let mut expired = HashSet::new();
        for successor in self.action(id).unwrap().successors().clone() {
            if self.action_mut(&successor).unwrap().satisfy(id) {
                ready.insert(successor);
            }
        }
        for dependent in self.action(id).unwrap().dependents().clone() {
            self.action_mut(&dependent).unwrap().expire();
            expired.insert(dependent);
        }
        (ready, expired)
    }

    pub fn wrap(&mut self, id: &ID) -> (HashSet<ID>, HashSet<ID>) {
        self.events.push(format!("{}  WRAP  {}", timestamp(), id));
        self.action_mut(id).unwrap().wrap();
        self.satisfy(id)
    }

    pub fn skip(&mut self, id: &ID) -> (HashSet<ID>, HashSet<ID>) {
        self.events.push(format!("{}  SKIP  {}", timestamp(), id));
        self.satisfy(id)
    }

    pub fn finish(&mut self) {
        async_write_to_file(
            Path::new(&self.log_dir).join("events.log").to_str().unwrap().to_string(),
            self.events.clone(),
            "Failed to write block event log to output file");
        self.events.clear();
    }
}
