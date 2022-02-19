use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use iced::{Column, Command, Element, Length, Row, Text, button, Align};
use iced_native::Space;
use serde::{Serialize, Deserialize};

use crate::block::Block;
use crate::comm::{Message, Value};
use crate::config::Config;
use crate::dispatch::Dispatcher;
use crate::style::{self, button, TEXT_LARGE, TEXT_XLARGE};
use crate::util::timestamp;
use crate::gui::GUI;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    title: String,
    version: String,
    #[serde(default, skip_serializing)]
    description: String,
    #[serde(default)]
    configuration: Config,
    #[serde(default)]
    blocks: Vec<Block>,
    #[serde(skip)]
    progress: Vec<bool>,
    #[serde(skip)]
    dispatcher: Option<Dispatcher>,
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    root_dir: String,
    #[serde(skip)]
    log_dir: String,
    #[serde(default)]
    gui: GUI,
    #[serde(skip)]
    events: Vec<String>,
    #[serde(skip)]
    active_block: Option<usize>,
}

#[derive(Debug, Clone)]
enum State {
    Startup {
        handles: [button::State; 2]
    },
    Configure,
    Selection {
        handles: [button::State; 64],
    },
    Starting {
        wait_for: u16,
    },
    Started,
}

impl Default for State {
    fn default() -> Self {
        State::Startup {
            handles: [button::State::new(); 2]
        }
    }
}

impl Task {
    pub fn new(task_dir: PathBuf) -> Self {
        let file = task_dir.join("task.yml");
        let file = File::open(file)
            .expect("Failed to open YAML file");
        let mut task: Task = serde_yaml::from_reader(file)
            .expect("Failed to read YAML file.");

        if task.description.starts_with("~") {
            let file = task_dir.join(&task.description[1..]);
            let mut file = File::open(file)
                .expect("Failed to open task description file");
            task.description = String::new();
            file.read_to_string(&mut task.description)
                .expect("Failed to read task description file");
        }

        let name = format!("session-{}", timestamp());
        task.log_dir = task_dir.join("output").join(name).to_str().unwrap().to_string();
        std::fs::create_dir_all(&task.log_dir)
            .expect("Failed to create output directory for task");

        for (i, block) in task.blocks.iter_mut().enumerate() {
            block.init(i+1, &task_dir);
        }
        task.progress = vec![false; task.blocks.len()];

        task.root_dir = task_dir.to_str().unwrap().to_string();
        task
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        let state = &mut self.state;
        let is_active = self.dispatcher.is_some()
            && self.dispatcher.as_ref().unwrap().is_active();

        match message {
            Message::SetComms(writer) => {
                if self.has_dispatcher() {
                    panic!("Tried to set up two dispatchers simultaneously");
                }
                self.dispatcher = Some(Dispatcher::new(writer));
                Command::none()
            }
            Message::Query(from, key) => {
                let response = Message::QueryResponse(
                    from,
                    match key.as_str() {
                        "task_dir" => {
                            self.root_dir.clone()
                        },
                        string if string.starts_with("config::") => {
                            self.configuration.query(&key[8..])
                        },
                        _ => panic!("Invalid query key: {}", key),
                    });
                self.dispatcher.as_mut().unwrap().update(response)
            }
            Message::UIEvent(code, value) => {
                match (state, code, value.clone()) {
                    (State::Startup { .. }, 0x01, _) => {
                        self.state = State::Configure;
                        Command::none()
                    }
                    (State::Startup { .. }, 0x02, _) => {
                        self.state = State::Selection {
                            handles: [button::State::new(); 64],
                        };
                        let file = File::create(Path::new(&self.log_dir).join("task.log")).unwrap();
                        serde_yaml::to_writer(file, &self);
                        Command::none()
                    }
                    (State::Configure, 0x01, _) => {
                        self.configuration.reset();
                        self.state = State::Startup {
                            handles: [button::State::new(); 2]
                        };
                        Command::none()
                    }
                    (State::Configure, 0x02, _) => {
                        self.configuration.reset();
                        Command::none()
                    }
                    (State::Configure, 0x03, _) => {
                        self.state = State::Selection {
                            handles: [button::State::new(); 64],
                        };
                        let file = File::create(Path::new(&self.log_dir).join("task.log")).unwrap();
                        serde_yaml::to_writer(file, &self);
                        Command::none()
                    }
                    (State::Configure, _, _) => {
                        self.configuration.update(code, value);
                        Command::none()
                    }
                    (State::Selection { .. }, i, Value::Null) => {
                        self.state = State::Starting {
                            wait_for: 30
                        };
                        Command::perform(async {
                            std::thread::sleep(Duration::from_millis(100));
                        }, move |()| Message::UIEvent(i, Value::Integer(29)))
                    }
                    (State::Starting { .. }, i, Value::Integer(0)) => {
                        self.state = State::Started;
                        self.execute(i as usize)
                    }
                    (State::Starting { wait_for, ..}, i, Value::Integer(t)) => {
                        *wait_for = t.clone() as u16;
                        Command::perform(async {
                            std::thread::sleep(Duration::from_millis(100));
                        }, move |()| Message::UIEvent(i, Value::Integer(t - 1)))
                    }
                    (State::Started { .. }, _, _) if is_active => {
                        self.dispatcher.as_mut().unwrap()
                            .update(Message::UIEvent(code, value))
                    }
                    _ => Command::none(),
                }
            }
            Message::Code(..) |
            Message::Value(..) |
            Message::KeyPress(..) |
            Message::ActionComplete(..) => {
                self.dispatcher.as_mut().unwrap().update(message)
            }
            Message::Interrupt => {
                if let Some(block) = self.active_block.take() {
                    self.events.push(format!("{}  INTERRUPT  {}", timestamp(), block));
                    let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
                    serde_yaml::to_writer(file, &self.events);
                }
                self.state = State::Selection {
                    handles: [button::State::new(); 64],
                };
                self.dispatcher.as_mut().unwrap().update(message)
            }
            Message::BlockComplete => {
                self.state = State::Selection {
                    handles: [button::State::new(); 64],
                };
                if let Some(block) = self.active_block.take() {
                    self.events.push(format!("{}  COMPLETE  {}", timestamp(), block));
                    let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
                    serde_yaml::to_writer(file, &self.events);
                }
                self.progress[self.dispatcher.as_ref().unwrap().block_id()-1] = true;
                self.dispatcher.as_mut().unwrap().update(message)
            }
            _ => {
                panic!("Asked to relay invalid message type");
            }
        }
    }

    pub fn has_dispatcher(&self) -> bool {
        self.dispatcher.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.dispatcher.is_some() && self.dispatcher.as_ref().unwrap().is_active()
    }

    pub fn execute<'b>(&mut self, block: usize) -> Command<Message> {
        if block == 0 {
            panic!("Block indexing starts from 1")
        }
        if self.dispatcher.as_ref().unwrap().is_active() {
            panic!("Tried to start a new block when another one is still running");
        }
        self.active_block = Some(block);
        self.events.push(format!("{}  START  {}", timestamp(), block));
        let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
        serde_yaml::to_writer(file, &self.events);
        let block = self.blocks[block-1].clone().with_log_dir(&self.log_dir);
        self.dispatcher.as_mut().unwrap().init(block)
    }

    pub fn view(&mut self) -> Column<Message> {
        let alignment = self.gui().alignment();
        let horizontal_alignment = self.gui().horizontal_alignment();
        let state = &mut self.state;
        let is_active = self.dispatcher.is_some()
            && self.dispatcher.as_ref().unwrap().is_active();

        match state {
            State::Startup { handles: [h_config, h_start] } => {
                let e_config: Element<Message> = if self.configuration.is_static() {
                    Space::with_width(Length::Units(200))
                        .into()
                } else {
                    button(h_start, "Configure", TEXT_LARGE)
                        .on_press(Message::UIEvent(0x01, Value::Null))
                        .style(style::Button::Secondary)
                        .width(Length::Units(200))
                        .padding(15)
                        .into()
                };

                let e_start = button(h_config, "Start!", TEXT_LARGE)
                    .on_press(Message::UIEvent(0x02, Value::Null))
                    .style(style::Button::Primary)
                    .width(Length::Units(200))
                    .padding(15);

                Column::new()
                    .width(Length::Fill)
                    .push(Column::new()
                        .width(Length::Fill)
                        .spacing(40)
                        .align_items(alignment)
                        .push(Text::new("Instructions")
                            .size(TEXT_XLARGE)
                            .horizontal_alignment(horizontal_alignment))
                        .push(Text::new(&self.description)
                            .size(TEXT_LARGE)
                            .horizontal_alignment(horizontal_alignment)))
                    .push(Space::with_height(Length::Fill))
                    .push(Row::new()
                        .push(e_config)
                        .push(Space::with_width(Length::Fill))
                        .push(e_start))
                    .into()
            }

            State::Configure => {
                self.configuration.view()
            }

            State::Selection { handles, .. } => {
                let elements: Vec<_> = self
                    .blocks
                    .iter()
                    .enumerate()
                    .zip(&self.progress)
                    .zip(handles)
                    .map(|(((i, block), is_done), h)| {
                        button(h, &block.title(), TEXT_XLARGE)
                            .on_press(Message::UIEvent((i + 1) as u16, Value::Null))
                            .style(if *is_done { style::Button::Done } else { style::Button::Todo })
                            .width(Length::Units(200))
                            .padding(15)
                    })
                    .collect();

                let mut rows = Column::new()
                    .spacing(40)
                    .align_items(Align::Center);
                let mut controls = Row::new()
                    .spacing(60);
                for (i, e) in elements.into_iter().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        rows = rows.push(controls);
                        controls = Row::new()
                            .spacing(60);
                    }
                    controls = controls.push(e);
                }
                rows = rows.push(controls);

                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(60)
                    .align_items(Align::Center)
                    .push(Space::with_height(Length::Fill))
                    .push(Text::new("Choose a block to start:").size(TEXT_XLARGE))
                    .push(rows)
                    .push(Space::with_height(Length::Fill))
            }

            State::Starting { wait_for, .. } => {
                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_items(Align::Center)
                    .push(Space::with_height(Length::Fill))
                    .push(Text::new(
                        format!("Starting block in {}...", (*wait_for as f32/10.0).ceil() as u16))
                        .size(TEXT_XLARGE))
                    .push(Space::with_height(Length::Fill))
            }

            State::Started { .. } if is_active => {
                self.dispatcher.as_mut().unwrap().view()
            }

            _ => Column::new()
        }
    }

    pub fn title(&self) -> String {
        format!("{} | v{} | rust-{}", self.title, self.version, env::consts::OS)
    }

    pub fn gui(&self) -> GUI {
        self.gui.clone()
    }
}
