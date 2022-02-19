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

    pub fn init(&mut self, block: Block) -> Command<Message> {
        self.queue = HashSet::from_iter(block.actions());
        self.block = Some(block);
        self.next()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        if self.block.is_none() {
            return Command::none()
        }

        match &message {
            Message::Code(_, id, ..) |
            Message::Value(_, id, ..) |
            Message::QueryResponse(id, ..) => {
                self.block.as_mut().unwrap().update(id, message.clone())
            }
            Message::KeyPress(_) => {
                if let Some(id) = &self.monitor_kb {
                    self.block.as_mut().unwrap().update(id, message.clone())
                } else if let Some(id) = &self.foreground {
                    self.block.as_mut().unwrap().update(id, message.clone())
                } else {
                    Command::none()
                }
            }
            Message::UIEvent(..) => {
                if let Some(id) = &self.foreground {
                    self.block.as_mut().unwrap().update(id, message.clone())
                } else {
                    Command::none()
                }
            }
            Message::ActionComplete(id) => {
                self.complete(id.clone())
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

    pub fn complete(&mut self, id: ID) -> Command<Message> {
        if self.block.is_none() || self.complete.contains(&id) {
            return Command::none();
        }
        let block = self.block.as_mut().unwrap();

        self.active.remove(&id);
        self.complete.insert(id.clone());
        block.wrap(&id);
        for dependent in block.dependents(&id) {
            if self.active.contains(&dependent) {
                self.active.remove(&dependent);
                self.complete.insert(dependent.clone());
                block.wrap(&dependent);
            }
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
        self.next()
    }

    pub fn next(&mut self) -> Command<Message> {
        if self.queue.is_empty() && self.active.is_empty() {
            return Command::perform(async {}, |()| Message::BlockComplete)
        }
        let block = self.block.as_mut().unwrap();

        let mut ready: HashSet<_> = self.queue
            .iter()
            .filter(|id| block.is_ready(id, &self.complete).unwrap_or(false))
            .cloned()
            .collect();

        let mut done = false;
        while !done {
            done = true;
            for id in ready.clone().iter() {
                for dependent in block.dependents(id) {
                    if self.queue.contains(&dependent) &&
                        !ready.contains(&dependent) &&
                        block.is_ready(&dependent, &self.complete).unwrap_or(true)
                    {
                        ready.insert(dependent);
                        done = false;
                    }
                }
            }
        }

        if self.active.is_empty() && !self.queue.is_empty() && ready.is_empty() {
            panic!("Action queue is not empty, but none ready to start")
        }

        let mut commands = vec![];
        for id in ready {
            if block.has_view(&id) {
                self.foreground = Some(id.clone());
            }
            if block.has_background(&id) {
                self.background = Some(id.clone());
            }
            if block.captures_keystrokes(&id) {
                self.monitor_kb = Some(id.clone());
            }
            self.queue.remove(&id);
            let command = block.execute(&id, self.writer.clone());
            self.active.insert(id);
            commands.push(command);
        }
        Command::batch(commands)
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
