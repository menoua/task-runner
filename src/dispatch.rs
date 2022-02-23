use std::collections::HashSet;
use iced::{Command, Column};

use crate::action::ID;
use crate::block::Block;
use crate::comm::{Message, Sender};
use crate::global::Global;

#[derive(Debug)]
pub struct Dispatcher {
    writer: Sender,
    block: Option<Block>,
    queue: HashSet<ID>,
    active: HashSet<ID>,
    complete: HashSet<ID>,
    foreground: Option<ID>,
    background: Option<ID>,
    monitor_kb: Option<ID>,
}

impl Dispatcher {
    pub fn new(writer: Sender) -> Self {
        Dispatcher {
            writer,
            block: None,
            queue: HashSet::new(),
            active: HashSet::new(),
            complete: HashSet::new(),
            foreground: None,
            background: None,
            monitor_kb: None,
        }
    }

    pub fn block_id(&self) -> usize {
        self.block.as_ref().unwrap().id()
    }

    pub fn is_active(&self) -> bool {
        self.block.is_some()
    }

    pub fn init(&mut self, block: Block, global: &Global) -> Command<Message> {
        self.queue = HashSet::from_iter(block.actions());
        self.block = Some(block);
        self.next(HashSet::from(["entry".to_string()]), global)
    }

    pub fn update(&mut self, message: Message, global: &Global) -> Command<Message> {
        if self.block.is_none() {
            return Command::none()
        }

        match &message {
            Message::Code(_, id, ..) |
            Message::Value(_, id, ..) |
            Message::QueryResponse(id, ..) => {
                self.block.as_mut().unwrap().update(id, message.clone(), global)
            }
            Message::KeyPress(_) => {
                if let Some(id) = &self.monitor_kb {
                    self.block.as_mut().unwrap().update(id, message.clone(), global)
                } else if let Some(id) = &self.foreground {
                    self.block.as_mut().unwrap().update(id, message.clone(), global)
                } else {
                    Command::none()
                }
            }
            Message::UIEvent(..) => {
                if let Some(id) = &self.foreground {
                    self.block.as_mut().unwrap().update(id, message.clone(), global)
                } else {
                    Command::none()
                }
            }
            Message::ActionComplete(id) => {
                self.complete(id.clone(), global)
            }
            Message::Interrupt |
            Message::BlockComplete => {
                if self.block.is_some() {
                    self.wrap_unfinished();
                    self.block = None;
                    self.queue.clear();
                    self.active.clear();
                    self.foreground = None;
                    self.complete.clear();
                }
                Command::none()
            }
            _ => panic!("Invalid message type for relaying")
        }
    }

    pub fn complete(&mut self, id: ID, global: &Global) -> Command<Message> {
        if self.block.is_none() || self.complete.contains(&id) {
            return Command::none();
        }
        let block = self.block.as_mut().unwrap();

        let mut ready = HashSet::new();
        let mut expired = HashSet::from([id]);
        while !expired.is_empty() {
            let mut new_expired = HashSet::new();
            for id in expired {
                if self.active.contains(&id) {
                    self.active.remove(&id);
                    self.complete.insert(id.clone());
                    let (ready2, expired2) = block.wrap(&id);
                    ready.extend(ready2);
                    new_expired.extend(expired2);
                }
            }
            expired = new_expired;
        }

        if let Some(id) = &self.foreground {
            if self.complete.contains(id) { self.foreground = None; }
        }
        if let Some(id) = &self.background {
            if self.complete.contains(id) { self.background = None; }
        }
        if let Some(id) = &self.monitor_kb {
            if self.complete.contains(id) { self.monitor_kb = None; }
        }
        self.next(ready, global)
    }

    pub fn next(&mut self, mut ready: HashSet<ID>, global: &Global) -> Command<Message> {
        let block = self.block.as_mut().unwrap();
        let mut commands = vec![];
        while !ready.is_empty() {
            let mut new_ready = HashSet::new();
            for id in ready {
                if block.is_expired(&id).unwrap_or(false) {
                    let mut expired = HashSet::from([id.clone()]);
                    while !expired.is_empty() {
                        for x in expired {
                            self.queue.remove(&x);
                            self.complete.insert(x);
                        }
                        let (ready2, expired2) = block.skip(&id);
                        new_ready.extend(ready2);
                        expired = expired2;
                    }
                } else {
                    if block.has_view(&id) {
                        self.foreground = Some(id.clone());
                    }
                    if block.has_background(&id) {
                        self.background = Some(id.clone());
                    }
                    if block.captures_keystrokes(&id) {
                        self.monitor_kb = Some(id.clone());
                    }
                    for dep in block.dependents(&id).to_owned() {
                        if block.is_ready(&dep).unwrap_or(true) {
                            new_ready.insert(dep);
                        }
                    }
                    self.queue.remove(&id);
                    let command = block.execute(&id, self.writer.clone(), global);
                    self.active.insert(id);
                    commands.push(command);
                }
            }
            ready = new_ready;
        }

        if !commands.is_empty() {
            Command::batch(commands)
        } else if !self.active.is_empty() {
            Command::none()
        } else if self.queue.is_empty() {
            Command::perform(async {}, |()| Message::BlockComplete)
        } else {
            panic!("Arrived at a deadlock; unable to reach some actions")
        }
    }

    pub fn wrap_unfinished(&mut self) {
        let block = self.block.as_mut().unwrap();
        for action in &self.active {
            block.wrap(action);
        }
        block.finish();
    }

    pub fn view(&mut self, global: &Global) -> Column<Message> {
        if let Some(id) = &self.foreground {
            self.block.as_mut().unwrap().view(id, global)
        } else if let Some(id) = &self.background {
            self.block.as_mut().unwrap().background(id)
        } else {
            Column::new()
        }
    }

    pub fn active_title(&self) -> String {
        if let Some(block) = &self.block {
            block.title()
        } else {
            "".to_string()
        }
    }
}
