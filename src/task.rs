use std::env;
use std::fmt::Debug;
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
use crate::style::{self, button};
use crate::util::{resource, timestamp};
use crate::global::Global;

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
    #[serde(default)]
    global: Global,
    #[serde(skip)]
    progress: Vec<bool>,
    #[serde(skip)]
    dispatcher: Option<Dispatcher>,
    #[serde(skip)]
    state: State,
    #[serde(skip)]
    log_dir: String,
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
    Configure {
        config: Config,
    },
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
    pub fn new(task_dir: PathBuf) -> Result<Self, String> {
        let file = task_dir.join("task.yml");
        let file = File::open(&file)
            .or(Err(format!("Failed to open YAML file: {:?}", file)))?;
        let mut task: Task = serde_yaml::from_reader(file)
            .or_else(|e| Err(format!(
                "Failed to read YAML file at line {}: {}",
                e.location().unwrap().line(), e)))?;

        if task.description.starts_with("<") {
            let file = resource(&task_dir, &task.description[1..].trim())?;
            let mut file = File::open(file)
                .or(Err("Failed to open task description file".to_string()))?;
            task.description.clear();
            file.read_to_string(&mut task.description)
                .or(Err("Failed to read task description file".to_string()))?;
        }

        let name = format!("session-{}", timestamp());
        task.log_dir = task_dir.join("output")
            .join(name).to_str().unwrap().to_string();
        std::fs::create_dir_all(&task.log_dir)
            .or(Err("Failed to create output directory for task".to_string()))?;

        for (i, block) in task.blocks.iter_mut().enumerate() {
            block.init(i+1, &task_dir)?;
        }
        task.progress = vec![false; task.blocks.len()];

        task.global.set_dir(task_dir.to_str().unwrap());
        Ok(task)
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
            Message::Query(_from, _key) => {
                // let response = Message::QueryResponse(
                //     from,
                //     match key.as_str() {
                //         _ => panic!("Invalid query key: {}", key),
                //     });
                // self.dispatcher.as_mut().unwrap().update(response, &self.global)
                Command::none()
            }
            Message::UIEvent(code, value) => {
                match (state, code, value.clone()) {
                    (State::Startup { .. }, 0x01, _) => {
                        self.state = State::Configure {
                            config: self.configuration.clone(),
                        };
                        Command::none()
                    }
                    (State::Startup { .. }, 0x02, _) => {
                        self.state = State::Selection {
                            handles: [button::State::new(); 64],
                        };
                        self.global.set_config(&self.configuration);
                        let file = File::create(Path::new(&self.log_dir).join("task.log")).unwrap();
                        serde_yaml::to_writer(file, &self)
                            .expect("Failed to write task configuration log to file");
                        Command::none()
                    }
                    (State::Configure { .. }, 0x01, _) => {
                        self.state = State::Startup {
                            handles: [button::State::new(); 2]
                        };
                        Command::none()
                    }
                    (State::Configure { config, .. }, 0x02, _) => {
                        *config = self.configuration.clone();
                        Command::none()
                    }
                    (State::Configure { config, .. }, 0x03, _) => {
                        self.configuration = config.clone();
                        self.global.set_config(&self.configuration);
                        self.state = State::Selection {
                            handles: [button::State::new(); 64],
                        };
                        let file = File::create(Path::new(&self.log_dir).join("task.log")).unwrap();
                        serde_yaml::to_writer(file, &self)
                            .expect("Failed to write task configuration log to file");
                        Command::none()
                    }
                    (State::Configure { config, .. }, _, _) => {
                        config.update(code, value);
                        Command::none()
                    }
                    (State::Selection { .. }, i, Value::Null) => {
                        self.state = State::Starting {
                            wait_for: 3000
                        };
                        Command::perform(async {
                            std::thread::sleep(Duration::from_millis(100));
                        }, move |()| Message::UIEvent(i, Value::Integer(2900)))
                    }
                    (State::Starting { .. }, i, Value::Integer(0)) => {
                        self.state = State::Started;
                        self.execute(i as usize)
                    }
                    (State::Starting { wait_for, ..}, i, Value::Integer(t)) => {
                        *wait_for = t.clone() as u16;
                        Command::perform(async {
                            std::thread::sleep(Duration::from_millis(100));
                        }, move |()| Message::UIEvent(i, Value::Integer(t - 100)))
                    }
                    (State::Started { .. }, _, _) if is_active => {
                        self.dispatcher.as_mut().unwrap()
                            .update(Message::UIEvent(code, value), &self.global)
                    }
                    _ => Command::none(),
                }
            }
            Message::Code(..) |
            Message::Value(..) |
            Message::KeyPress(..) |
            Message::ActionComplete(..) => {
                self.dispatcher.as_mut().unwrap().update(message, &self.global)
            }
            Message::Interrupt => {
                match state {
                    State::Startup { .. } |
                    State::Selection { .. } => {
                        Command::none()
                    },
                    State::Configure { .. } => {
                        self.state = State::Startup {
                            handles: [button::State::new(); 2]
                        };
                        Command::none()
                    }
                    State::Starting { .. } => {
                        self.state = State::Selection {
                            handles: [button::State::new(); 64],
                        };
                        Command::none()
                    }
                    State::Started => {
                        if let Some(block) = self.active_block.take() {
                            self.events.push(format!("{}  INTERRUPT  {}", timestamp(), block));
                            let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
                            serde_yaml::to_writer(file, &self.events)
                                .expect("Failed to write interrupted block event log to file");

                            self.state = State::Selection {
                                handles: [button::State::new(); 64],
                            };
                            self.dispatcher.as_mut().unwrap().update(message, &self.global)
                        } else {
                            Command::none()
                        }
                    }
                }
            }
            Message::BlockComplete => {
                self.state = State::Selection {
                    handles: [button::State::new(); 64],
                };
                if let Some(block) = self.active_block.take() {
                    self.events.push(format!("{}  COMPLETE  {}", timestamp(), block));
                    let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
                    serde_yaml::to_writer(file, &self.events)
                        .expect("Failed to write completed block event log to file");
                }
                self.progress[self.dispatcher.as_ref().unwrap().block_id()-1] = true;
                self.dispatcher.as_mut().unwrap().update(message, &self.global)
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
        self.global.reset_io();
        self.active_block = Some(block);
        self.events.push(format!("{}  START  {}", timestamp(), block));
        let file = File::create(Path::new(&self.log_dir).join("events.log")).unwrap();
        serde_yaml::to_writer(file, &self.events)
            .expect("Failed to write block start event to file");
        let block = self.blocks[block-1].clone().with_log_dir(&self.log_dir);
        self.dispatcher.as_mut().unwrap().init(block, &self.global)
    }

    pub fn view(&mut self) -> Column<Message> {
        let state = &mut self.state;
        let is_active = self.dispatcher.is_some()
            && self.dispatcher.as_ref().unwrap().is_active();

        match state {
            State::Startup { handles: [h_config, h_start] } => {
                let e_config: Element<Message> = if self.configuration.is_static() {
                    Space::with_width(Length::Units(200))
                        .into()
                } else {
                    button(
                        h_start,
                        "Configure",
                        self.global.text_size("LARGE"))
                        .on_press(Message::UIEvent(0x01, Value::Null))
                        .style(style::Button::Secondary)
                        .width(Length::Units(200))
                        .padding(15)
                        .into()
                };

                let e_start = button(
                    h_config,
                    "Start!",
                    self.global.text_size("LARGE"))
                    .on_press(Message::UIEvent(0x02, Value::Null))
                    .style(style::Button::Primary)
                    .width(Length::Units(200))
                    .padding(15);

                Column::new()
                    .width(Length::Fill)
                    .push(Column::new()
                        .width(Length::Fill)
                        .spacing(40)
                        .align_items(self.global.alignment())
                        .push(Text::new("Instructions")
                            .size(self.global.text_size("XLARGE"))
                            .horizontal_alignment(self.global.horizontal_alignment()))
                        .push(Text::new(&self.description)
                            .size(self.global.text_size("LARGE"))
                            .horizontal_alignment(self.global.horizontal_alignment())))
                    .push(Space::with_height(Length::Fill))
                    .push(Row::new()
                        .push(e_config)
                        .push(Space::with_width(Length::Fill))
                        .push(e_start))
                    .into()
            }

            State::Configure { config,.. } => {
                config.view(&self.global)
            }

            State::Selection { handles, .. } => {
                let elements: Vec<_> = self
                    .blocks
                    .iter()
                    .enumerate()
                    .zip(&self.progress)
                    .zip(handles)
                    .map(|(((i, block), is_done), h)| {
                        button(
                            h,
                            &block.title(),
                            self.global.text_size("XLARGE"))
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
                    .spacing(60)
                    .align_items(Align::Center);
                for (i, e) in elements.into_iter().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        rows = rows.push(controls);
                        controls = Row::new()
                            .spacing(60)
                            .align_items(Align::Center);
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
                    .push(Text::new("Choose a block to start:")
                        .size(self.global.text_size("XLARGE")))
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
                        format!("Starting block in {}...", (*wait_for+999)/1000))
                        .size(self.global.text_size("XLARGE")))
                    .push(Space::with_height(Length::Fill))
            }

            State::Started { .. } if is_active => {
                self.dispatcher.as_mut().unwrap().view(&self.global)
            }

            _ => Column::new()
        }
    }

    pub fn title(&self) -> String {
        format!("{} | v{} | rust-{}", self.title, self.version, env::consts::OS)
    }

    pub fn global(&self) -> &Global {
        &self.global
    }
}
