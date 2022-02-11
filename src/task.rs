use iced::{
    button, Align, Application, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, Row, Space, Subscription, Text,
};
use iced_native::keyboard::KeyCode;
use iced_native::subscription;
use std::env;
use std::time::{Duration, Instant};

use crate::block::Block;
use crate::config::Config;
use crate::error::Error;
use crate::logger::{Event, Logger, Reaction, Response};
use crate::question::Question;
use crate::style::{self, button, TEXT_LARGE, TEXT_NORMAL, TEXT_XLARGE};

use state::State;

pub struct Task<T>
where
    T: Block, // + 'static
{
    title: String,
    version: String,
    config: T::Config,
    blocks: Vec<T>,
    progress: Vec<bool>,
    state: State<T>,
    logger: Logger<T>,
    description: String,
}

mod state {
    use super::*;
    use crate::block::Communication;
    use std::fmt::Formatter;

    // #[derive(Debug)]
    pub enum State<T: Block> {
        Startup {
            configure: button::State,
            start: button::State,
        },

        Configure {
            options: T::Config,
            handles: Vec<Vec<button::State>>,
            // elements: Vec<Vec<Button<'a, Message<T>>>>,
            cancel: button::State,
            revert: button::State,
            apply: button::State,
        },

        Selection {
            handles: Vec<button::State>,
            // elements: Vec<Button<'a, Message<T>>>,
        },

        Countdown {
            block: usize,
            target: Instant,
            remaining: Duration,
            cancel: button::State,
        },

        Active {
            block: usize,
            last_esc: Instant,
            comm: Communication,
        },

        Query {
            block: usize,
            current: T::Question,
            queue: Vec<T::Question>,
            handles: Vec<button::State>,
            submit: button::State,
        },
    }

    impl<T: Block> std::fmt::Display for State<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                State::Startup { .. } => write!(f, "Startup"),
                State::Configure { .. } => write!(f, "Configure"),
                State::Selection { .. } => write!(f, "Selection"),
                State::Countdown { block, .. } => write!(f, "Countdown (block={})", block),
                State::Active { block, .. } => write!(f, "Active (block={})", block),
                State::Query { block, .. } => write!(f, "Query (block={})", block),
            }
        }
    }

    pub fn startup<T: Block>() -> State<T> {
        State::Startup {
            configure: button::State::new(),
            start: button::State::new(),
        }
    }

    pub fn configure<T: Block>(config: &T::Config) -> State<T> {
        let options = config.clone();

        let handles: Vec<_> = T::Config::keys()
            .into_iter()
            .map(|k| {
                T::Config::values(k)
                    .into_iter()
                    .map(|_| button::State::new())
                    .collect()
            })
            .collect();

        State::Configure {
            options,
            handles,
            cancel: button::State::new(),
            revert: button::State::new(),
            apply: button::State::new(),
        }
    }

    pub fn selection<T: Block>(blocks: &[T]) -> State<T> {
        let handles: Vec<_> = blocks.into_iter().map(|_| button::State::new()).collect();

        State::Selection {
            handles,
            // elements,
        }
    }

    pub fn countdown<'a, T: Block>(block: usize) -> State<T> {
        State::Countdown {
            block,
            target: Instant::now() + Duration::from_secs(5),
            remaining: Duration::from_secs(5),
            cancel: button::State::new(),
        }
    }

    pub fn active<'a, T: Block>(block: usize) -> (State<T>, Communication) {
        let (comm_task, comm_block) = Communication::two_way();

        (
            State::Active {
                block,
                last_esc: Instant::now() - Duration::from_secs(10),
                comm: comm_task,
            },
            comm_block,
        )
    }

    pub fn query<'a, T: Block>(block: usize, mut questionnaire: Vec<T::Question>) -> State<T> {
        State::Query {
            block,
            current: questionnaire.remove(0),
            queue: questionnaire,
            handles: vec![],
            submit: button::State::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message<T: Block> {
    ChangeConfig,
    CancelConfig,
    RevertConfig,
    UpdateConfig(String, <T::Config as Config>::Item),
    StartTask,
    StartBlock(usize),
    AdvanceTimer(Instant),
    StopTimer,
    Reaction(KeyCode),
    BlockInterrupt,
    BlockFinished(Result<(), Error>),
    UpdateResponse(T::Answer),
    SubmitResponse(Result<(), Error>),
}

impl<T> Task<T>
where
    T: Block,
{
    pub fn new() -> Self {
        Task {
            title: String::from("Default"),
            version: String::from("0.0.0"),
            config: T::Config::default(),
            blocks: vec![],
            progress: vec![],
            state: state::startup(),
            logger: Logger::new(),
            description: String::from("Default"),
        }
    }

    pub fn set_title(mut self, title: &str) -> Self {
        self.title = String::from(title);
        self
    }

    pub fn set_version(mut self, version: &str) -> Self {
        self.version = String::from(version);
        self
    }

    pub fn set_config(mut self, config: T::Config) -> Self {
        self.config = config;
        self
    }

    pub fn set_blocks(mut self, blocks: Vec<T>) -> Self {
        self.blocks = blocks;
        self.progress = vec![false; self.blocks.len()];
        self
    }

    pub fn set_description(mut self, description: &str) -> Self {
        self.description = String::from(description);
        self
    }
}

pub type Builder<T> = Option<Box<dyn FnOnce() -> Task<T>>>;

impl<T: Block> Application for Task<T> {
    type Executor = iced::executor::Default;
    type Message = Message<T>;
    type Flags = Builder<T>;

    fn new(flags: Self::Flags) -> (Task<T>, Command<Self::Message>) {
        let task = match flags {
            Some(builder) => builder(),
            None => Task::new(),
        };

        println!(
            ">> {} | v{} | rust-{}",
            task.title,
            task.version,
            env::consts::OS
        );
        (task, Command::none())
    }

    fn title(&self) -> String {
        let subtitle = match self.state {
            State::Startup { .. } => "Welcome",
            State::Configure { .. } => "Configuration",
            State::Selection { .. } => "Block selection",
            State::Countdown { .. } => "Block countdown",
            State::Active { .. } => "Block running",
            State::Query { .. } => "Question",
        };

        format!("{} (v{}) - {}", self.title, self.version, subtitle)
    }

    fn update(&mut self, message: Self::Message, _: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::ChangeConfig => {
                self.state = state::configure(&self.config);
                Command::none()
            }

            Message::CancelConfig => {
                self.state = state::startup();
                Command::none()
            }

            Message::RevertConfig => {
                if let State::Configure { options, .. } = &mut self.state {
                    *options = self.config.clone();
                }
                Command::none()
            }

            Message::UpdateConfig(key, value) => {
                if let State::Configure { options, .. } = &mut self.state {
                    options.update(&key, value);
                }
                Command::none()
            }

            Message::StartTask => {
                if let State::Configure { options, .. } = &self.state {
                    self.config = options.clone();
                }

                self.logger.log_event(Event::Init {
                    task: self.title.clone(),
                    version: self.version.clone(),
                    sess_id: self.logger.sess_id(),
                    config: self.config.clone(),
                });

                self.state = state::selection(&self.blocks);
                Command::none()
            }

            Message::StartBlock(block) => {
                if let State::Selection { .. } = self.state {
                    self.logger.log_event(Event::BlockStart {
                        id: self.blocks[block].id(),
                    });

                    self.state = state::countdown(block);
                }
                Command::none()
            }

            Message::AdvanceTimer(now) => {
                let mut timeout = false;
                let mut block_num = 0 as usize;

                if let State::Countdown {
                    block,
                    target,
                    remaining,
                    ..
                } = &mut self.state
                {
                    if now < *target {
                        *remaining = *target - now;
                    } else {
                        timeout = true;
                        block_num = *block;
                    }
                }

                if timeout {
                    let (state, comm) = state::active(block_num);
                    self.state = state;

                    let id = String::from(self.blocks[block_num].id());
                    let config = self.config.clone();
                    Command::perform(
                        async move { T::run(id, config, comm) },
                        Message::BlockFinished,
                    )
                } else {
                    Command::none()
                }
            }

            Message::StopTimer => {
                if let State::Countdown { .. } = self.state {
                    self.state = state::selection(&self.blocks);
                }
                Command::none()
            }

            Message::Reaction(key_code) => {
                if let State::Active { block, .. } = self.state {
                    self.logger
                        .log_reaction(Reaction::new(self.blocks[block].id(), key_code));
                    eprintln!("Key press logged.")
                }
                Command::none()
            }

            Message::BlockInterrupt => {
                if let State::Active {
                    block,
                    last_esc,
                    comm,
                    ..
                } = &mut self.state
                {
                    let now = Instant::now();

                    if now.duration_since(*last_esc) < Duration::from_millis(250) {
                        comm.send().unwrap();
                        self.logger.log_event(Event::BlockEnd {
                            id: self.blocks[*block].id(),
                            success: false,
                        });

                        eprintln!("Block interrupted!");
                        self.state = state::selection(&self.blocks);
                    } else {
                        *last_esc = now;
                    }
                }
                Command::none()
            }

            Message::BlockFinished(Ok(_)) => {
                if let State::Active { block, .. } = self.state {
                    self.logger.log_event(Event::BlockEnd {
                        id: self.blocks[block].id(),
                        success: true,
                    });

                    let id = self.blocks[block].id();
                    let questionnaire = T::questionnaire(&id, &self.config);

                    if questionnaire.len() > 0 {
                        self.state = state::query(block, questionnaire);
                    } else {
                        self.state = state::selection(&self.blocks);
                    }
                }
                Command::none()
            }

            Message::BlockFinished(Err(_)) => {
                if let State::Active { block, .. } = self.state {
                    self.logger.log_event(Event::BlockEnd {
                        id: self.blocks[block].id(),
                        success: false,
                    });

                    eprintln!("Block interrupted!");
                    self.state = state::selection(&self.blocks);
                }
                Command::none()
            }

            Message::UpdateResponse(value) => {
                if let State::Query { current, .. } = &mut self.state {
                    current.update(value)
                }
                Command::none()
            }

            Message::SubmitResponse(Ok(_)) => {
                if let State::Query {
                    block,
                    current,
                    queue,
                    ..
                } = &mut self.state
                {
                    self.logger
                        .log_response(Response::new(self.blocks[*block].id(), current.summary()));

                    if queue.len() > 0 {
                        self.state = state::query(*block, queue.clone());
                    } else {
                        self.state = state::selection(&self.blocks);
                    }
                }
                Command::none()
            }

            Message::SubmitResponse(Err(_)) => {
                if let State::Query { .. } = self.state {
                    // progress[*block] = true;
                    self.state = state::selection(&self.blocks);
                }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message<T>> {
        use iced_native::keyboard::Event::KeyPressed;
        use iced_native::Event::Keyboard;

        match self.state {
            State::Countdown { .. } => Subscription::batch([
                iced::time::every(Duration::from_secs(1)).map(Message::AdvanceTimer),
                subscription::events_with(|event, _| match event {
                    Keyboard(keyboard_event) => match keyboard_event {
                        KeyPressed {
                            key_code: KeyCode::Escape,
                            ..
                        } => Some(Message::StopTimer),
                        _ => None,
                    },
                    _ => None,
                }),
            ]),

            State::Active { .. } => subscription::events_with(|event, _| match event {
                Keyboard(keyboard_event) => match keyboard_event {
                    KeyPressed {
                        key_code: KeyCode::Escape,
                        ..
                    } => Some(Message::BlockInterrupt),
                    KeyPressed { key_code, .. } => Some(Message::Reaction(key_code)),
                    _ => None,
                },
                _ => None,
            }),

            _ => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<Message<T>> {
        let content = match &mut self.state {
            State::Startup { configure, start } => Column::new()
                .width(Length::Units(600))
                .spacing(40)
                .push(Text::new("Instructions").size(TEXT_XLARGE))
                .push(Text::new(&self.description).size(TEXT_LARGE))
                .push(
                    Row::new()
                        .push(
                            button(configure, "Configure", TEXT_LARGE)
                                .on_press(Message::ChangeConfig)
                                .style(style::Button::Secondary)
                                .width(Length::Units(200)),
                        )
                        .push(Space::with_width(Length::Fill))
                        .push(
                            button(start, "Start!", TEXT_LARGE)
                                .on_press(Message::StartTask)
                                .style(style::Button::Primary)
                                .width(Length::Units(200)),
                        ),
                ),

            State::Configure {
                options,
                handles,
                cancel,
                revert,
                apply,
                ..
            } => {
                let elements: Vec<Vec<_>> = T::Config::keys()
                    .into_iter()
                    .zip(handles)
                    .map(|(k, hs)| {
                        T::Config::values(k)
                            .into_iter()
                            .zip(hs)
                            .map(|((l, v), h)| {
                                button(h, l, TEXT_NORMAL)
                                    .width(Length::Units(275))
                                    .style(if options.get(k) == v {
                                        style::Button::Active
                                    } else {
                                        style::Button::Inactive
                                    })
                                    .on_press(Message::UpdateConfig(String::from(k), v))
                            })
                            .collect()
                    })
                    .collect();

                let mut content = Column::new()
                    .width(Length::Units(600))
                    .spacing(60)
                    .align_items(Align::Start);

                for (k, es) in T::Config::keys().into_iter().zip(elements) {
                    let mut row = Row::new();
                    for (i, e) in es.into_iter().enumerate() {
                        if i > 0 {
                            row = row.push(Space::with_width(Length::Fill));
                        }
                        row = row.push(e);
                    }

                    content = content.push(
                        Column::new()
                            .spacing(20)
                            .push(
                                Text::new(format!("{}:", T::Config::description(k)))
                                    .size(TEXT_NORMAL),
                            )
                            .push(row),
                    );
                }

                content.push(
                    Row::new()
                        .push(
                            button(cancel, "Cancel", TEXT_LARGE)
                                .on_press(Message::CancelConfig)
                                .style(style::Button::Secondary)
                                .width(Length::Units(175)),
                        )
                        .push(Space::with_width(Length::Fill))
                        .push(
                            button(revert, "Revert", TEXT_LARGE)
                                .on_press(Message::RevertConfig)
                                .style(style::Button::Destructive)
                                .width(Length::Units(175)),
                        )
                        .push(Space::with_width(Length::Fill))
                        .push(
                            button(apply, "Start!", TEXT_LARGE)
                                .on_press(Message::StartTask)
                                .style(style::Button::Primary)
                                .width(Length::Units(175)),
                        ),
                )
            }

            State::Selection { handles, .. } => {
                let mut controls = Row::new();

                let elements: Vec<_> = self
                    .blocks
                    .iter()
                    .enumerate()
                    .zip(&self.progress)
                    .zip(handles)
                    .map(|(((i, block), is_done), h)| {
                        button(h, &block.title(), TEXT_XLARGE)
                            .on_press(Message::StartBlock(i))
                            .style(if *is_done {
                                style::Button::Done
                            } else {
                                style::Button::Todo
                            })
                            .width(Length::Units(175))
                            .padding(20)
                    })
                    .collect();

                for (i, e) in elements.into_iter().enumerate() {
                    if i > 0 {
                        controls = controls.push(Space::with_width(Length::Fill));
                    }
                    controls = controls.push(e);
                }

                Column::new()
                    .width(Length::Units(600))
                    .spacing(40)
                    .align_items(Align::Center)
                    .push(Text::new("Choose a block to start:").size(TEXT_XLARGE))
                    .push(controls)
            }

            State::Countdown {
                block,
                remaining,
                cancel,
                ..
            } => Column::new()
                .width(Length::Units(600))
                .align_items(Align::Center)
                .spacing(60)
                .push(
                    Text::new(format!(
                        "Starting block in {:?} seconds...",
                        remaining.as_secs_f32().round() as i32
                    ))
                    .size(TEXT_XLARGE),
                )
                .push(
                    Text::new(self.blocks[*block].description())
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .size(TEXT_XLARGE),
                )
                .push(
                    button(cancel, "Cancel", TEXT_NORMAL)
                        .on_press(Message::StopTimer)
                        .style(style::Button::Destructive)
                        .width(Length::Units(150)),
                ),

            State::Active { block, .. } => Column::new()
                .push(self.blocks[*block].view())
                .width(Length::Units(600))
                .align_items(Align::Center),

            State::Query {
                current, submit, ..
            } => Column::new()
                .width(Length::Units(600))
                .spacing(60)
                .align_items(Align::Center)
                .push(current.view())
                .push(
                    button(submit, "Submit", TEXT_LARGE)
                        .on_press(Message::SubmitResponse(Ok(())))
                        .width(Length::Units(400)),
                ),
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
