use std::collections::HashSet;
use std::fs::File;
use std::ops::RangeInclusive;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use iced::{image, Column, Length, Text, Align, button, Checkbox, TextInput, text_input, Space, Container, slider};
use iced_futures::Command;
use iced_native::Image;

use crate::comm::{Comm, Message, Receiver, Sender, Value};
use crate::sound::play_audio;
use crate::util::timestamp;
use crate::style::button;

use Question::*;
use crate::global::Global;

pub type ID = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    #[serde(default)]
    id: ID,
    #[serde(default, skip_serializing_if="Option::is_none")]
    with: Option<ID>,
    #[serde(default)]
    after: Option<HashSet<ID>>,
    #[serde(default, skip_serializing_if="std::ops::Not::not")]
    monitor_kb: bool,
    #[serde(skip)]
    keystrokes: Vec<String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    background: Option<String>,
    #[serde(skip)]
    background_image: Option<image::Handle>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    timeout: Option<u16>,
    // #[serde(skip)]
    // task_dir: String,
    #[serde(skip)]
    log_prefix: String,
    #[serde(skip)]
    comm: Option<Sender>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub enum Action {
    Instruction {
        prompt: String,
        #[serde(default="default::timer")]
        timer: u16,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        handle: Option<button::State>,
    },
    Nothing {
        #[serde(default="default::timer")]
        timer: u16,
        #[serde(default, flatten)]
        info: Info,
    },
    Audio {
        source: String,
        #[serde(default, flatten)]
        info: Info,
    },
    Question {
        list: Vec<Question>,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        handle: button::State,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Question {
    #[serde(serialize_with="serialize::question::single_choice")]
    SingleChoice {
        prompt: String,
        options: Vec<String>,
        #[serde(skip_deserializing)]
        answer: Option<usize>,
    },
    #[serde(serialize_with="serialize::question::multi_choice")]
    MultiChoice {
        prompt: String,
        options: Vec<String>,
        #[serde(skip_deserializing)]
        answer: Vec<bool>,
    },
    ShortAnswer {
        prompt: String,
        #[serde(skip_deserializing)]
        answer: String,
        #[serde(skip)]
        handles: [text_input::State; 1],
    },
    Slider {
        prompt: String,
        #[serde(default="default::slider_range")]
        range: RangeInclusive<f32>,
        #[serde(default="default::slider_step")]
        step: f32,
        #[serde(skip_deserializing)]
        answer: f32,
        #[serde(skip)]
        handles: [slider::State; 1],
    },
}

impl Question {
    pub fn init(&mut self) {
        match self {
            MultiChoice { options, answer, .. } => {
                *answer = vec![false; options.len()];
            }
            ShortAnswer { handles, .. } => {
                *handles = [text_input::State::new(); 1];
            }
            Slider { handles, .. } => {
                *handles = [slider::State::new(); 1];
            }
            _ => ()
        }
    }

    pub fn update(&mut self, value: Value) {
        match (self, value) {
            (SingleChoice { answer, .. }, Value::Integer(i)) => {
                *answer = Some(i as usize);
            }
            (MultiChoice { answer, .. }, Value::Integer(i)) => {
                answer[i as usize] = !answer[i as usize];
            }
            (ShortAnswer { answer, .. }, Value::String(s)) => {
                *answer = s;
            }
            (Slider { answer, .. }, Value::Float(f)) => {
                *answer = f;
            }
            _ => panic!("Invalid answer value type")
        }
    }
}

impl Action {
    pub fn init(
        &mut self,
        mut next_id: i32,
        last_action: &Option<ID>,
        task_dir: &Path
    ) -> (i32, Option<ID>) {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                if info.id.is_empty() {
                    info.id = next_id.to_string();
                    next_id += 1;
                }
                match (&info.after, &info.with) {
                    (None, None) => {
                        if let Some(last_id) = last_action {
                            info.after = Some(HashSet::from([last_id.clone()]));
                        } else {
                            info.after = Some(HashSet::new());
                        }
                    }
                    _ => (),
                }
                if let Some(file) = &info.background {
                    let file = task_dir.join(file);
                    assert!(file.exists(), "Background image file {:?} not found", file);
                    info.background_image = Some(image::Handle::from_path(file));
                }
            }
        }

        match self {
            Action::Instruction { timer, handle, .. } => {
                *handle = if *timer == 0 {
                    Some(button::State::new())
                } else {
                    None
                };
            }
            Action::Question { list, .. } => {
                for quest in list {
                    quest.init();
                }
            }
            _ => ()
        }

        (next_id, Some(self.id()))
    }

    pub fn id(&self) -> ID {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                info.id.clone()
            },
        }
    }

    pub fn set_id(&mut self, id: &ID) {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                info.id = id.clone();
            }
        }
    }

    pub fn is(&self, id: &str) -> bool {
        self.id() == id
    }

    pub fn with(&self) -> Option<ID> {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                info.with.clone()
            }
        }
    }

    pub fn after(&self) -> HashSet<ID> {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                if let Some(ids) = &info.after {
                    ids.clone()
                } else {
                    HashSet::new()
                }
            }
        }
    }

    pub fn is_ready(&self, complete: &HashSet<ID>) -> Option<bool> {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                if let Some(ids) = &info.after {
                    Some(ids.iter().all(|x| complete.contains(x)))
                } else {
                    None
                }
            }
        }
    }

    pub fn has_view(&self) -> bool {
        match self {
            Action::Nothing { .. } |
            Action::Audio { .. } => false,

            Action::Instruction { .. } |
            Action::Question { .. } => true,
        }
    }

    pub fn has_background(&self) -> bool {
        match self {
            Action::Instruction { info, .. } |
            Action::Question { info, .. } |
            Action::Nothing { info, .. } |
            Action::Audio { info, .. } => info.background.is_some()
        }
    }

    pub fn captures_keystrokes(&self) -> bool {
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => info.monitor_kb
        }
    }

    pub fn run(&mut self, writer: Sender, log_dir: &str, global: &Global) -> Command<Message> {
        match self {
            Action::Instruction { info, ..} |
            Action::Audio { info, ..} |
            Action::Nothing { info, .. } |
            Action::Question { info, ..} => {
                info.log_prefix = Path::new(log_dir)
                    .join(format!("action-{}-{}", info.id, timestamp()))
                    .to_str().unwrap().to_string();
            }
        }

        let mut commands = vec![];
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                if let Some(t) = info.timeout {
                    let id = self.id();
                    commands.push(Command::perform(async move {
                        std::thread::sleep(Duration::from_secs(t as u64));
                        Message::ActionComplete(id)
                    }, |msg| msg));
                }
            }
        }

        match self {
            Action::Instruction { timer, .. } => {
                if *timer > 0 {
                    let timer = timer.clone();
                    let rx = self.new_comm_link();
                    commands.push(Command::perform(
                        run::instruction(self.id(), (writer, rx), timer),
                        |msg| msg));
                }
            }
            Action::Audio { source, .. } => {
                let source = Path::new(global.dir()).join(source);
                let use_trigger = global.config().use_trigger();
                let stream_handle = global.io().audio_stream();

                let source = source.clone();
                let rx = self.new_comm_link();
                commands.push(Command::perform(
                    run::audio(self.id(), (writer, rx), source, use_trigger, stream_handle),
                    |msg| msg));
            }
            _ => (),
        }

        Command::batch(commands)
    }

    pub fn view(&mut self, global: &Global) -> Column<Message> {
        let id = self.id();
        match self {
            Action::Instruction { prompt, handle, .. } => {
                if let Some(handle) = handle {
                    let e_next = button(
                        handle,
                        "Next",
                        global.text_size("XLARGE"))
                        .on_press(Message::ActionComplete(id))
                        .width(Length::Units(400));

                    Column::new()
                        .width(Length::Fill)
                        .align_items(Align::Center)
                        .push(Space::with_height(Length::Fill))
                        .push(Text::new(prompt.clone())
                            .size(global.text_size("XLARGE"))
                            .horizontal_alignment(global.horizontal_alignment()))
                        .push(Space::with_height(Length::Fill))
                        .push(e_next)
                } else {
                    Column::new()
                        .width(Length::Fill)
                        .align_items(Align::Center)
                        .push(Space::with_height(Length::Fill))
                        .push(Text::new(prompt.clone())
                            .size(global.text_size("XLARGE"))
                            .horizontal_alignment(global.horizontal_alignment()))
                        .push(Space::with_height(Length::Fill))
                }
            }
            Action::Question { list: questions, handle, .. } => {
                let mut content = Column::new()
                    // .width(Length::Fill)
                    .spacing(40)
                    .align_items(Align::Start);
                for (i, quest) in questions.iter_mut().enumerate() {
                    content = content.push(view::question(quest, i, global));
                }

                let e_submit = button(
                    handle,
                    "Submit",
                    global.text_size("XLARGE"))
                    .on_press(Message::ActionComplete(id))
                    .width(Length::Units(400));

                Column::new()
                    // .width(Length::Fill)
                    .align_items(Align::Center)
                    .push(content)
                    .push(Space::with_height(Length::Fill))
                    .push(e_submit)
                    .into()
            }
            _ => panic!("Action does not have a view"),
        }
    }

    pub fn update(&mut self, message: Message, _global: &Global) -> Command<Message> {
        if let Message::KeyPress(key_code) = message {
            return match self {
                Action::Audio { info, .. } |
                Action::Instruction { info, .. } |
                Action::Nothing { info, .. } |
                Action::Question { info, .. } => {
                    info.keystrokes.push(format!("{}  {:?}", timestamp(), key_code));
                    Command::none()
                }
            };
        }

        match self {
            Action::Audio { info, .. } => {
                match message {
                    Message::QueryResponse(..) => {
                        info.comm.as_mut().unwrap().send(message.clone()).ok();
                        Command::none()
                    }
                    _ => {
                        panic!("{:?}", message);
                    }
                }
            }
            Action::Question { list, .. } => {
                match message {
                    Message::UIEvent(code, value) => {
                        list[(code - 0x01) as usize].update(value);
                        Command::none()
                    }
                    _ => {
                        panic!("{:?}", message);
                    }
                }
            }
            _ => panic!()
        }
    }

    pub fn background(&mut self) -> Column<Message> {
        let image = match self {
            Action::Instruction { info, .. } |
            Action::Question { info, .. } |
            Action::Nothing { info, .. } |
            Action::Audio { info, .. } => {
                Image::new(info.background_image.as_ref().unwrap().clone())
            }
        };

        Column::new()
            .push(Container::new(image)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y())
            .width(Length::Fill)
            .height(Length::Fill)
    }

    pub fn wrap(&self) {
        match self {
            Action::Audio { info, .. } |
            Action::Instruction { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                if info.monitor_kb {
                    let file = File::create(format!("{}.keypress", info.log_prefix)).unwrap();
                    serde_yaml::to_writer(file, &info.keystrokes)
                        .expect("Failed to write key presses to output file");
                }
                if let Some(comm) = &info.comm {
                    comm.send(Message::Wrap).ok();
                }
            }
        }
        match self {
            Action::Question { info, list, .. } => {
                let file = File::create(format!("{}.response", info.log_prefix)).unwrap();
                serde_yaml::to_writer(file, &list)
                    .expect("Failed to write question responses to output file");
            }
            _ => ()
        }
    }

    pub fn new_comm_link(&mut self) -> Receiver {
        let (tx, rx) = mpsc::channel();
        match self {
            Action::Instruction { info, .. } |
            Action::Audio { info, .. } |
            Action::Nothing { info, .. } |
            Action::Question { info, .. } => {
                info.comm = Some(tx);
            }
        }
        rx
    }
}

pub mod view {
    use iced::{Radio, Row};
    use super::*;

    pub fn question<'a>(quest: &'a mut Question, index: usize, global: &Global) -> Column<'a, Message> {
        match quest {
            Question::SingleChoice {
                prompt,
                options,
                answer
            } => {
                let mut row = Row::new()
                    // .width(Length::Fill)
                    .spacing(40);
                for i in 0..options.len() {
                    let ind = index.clone();
                    row = row.push(Radio::new(
                        i,
                        options[i].clone(),
                        answer.clone(),
                        move |_value| Message::UIEvent(
                            (0x01 + ind) as u16,
                            Value::Integer(i as i32)))
                        // .width(Length::Units(250))
                        .text_size(global.text_size("XLARGE"))
                        .size(global.text_size("LARGE")));
                }

                Column::new()
                    // .width(Length::Fill)
                    .align_items(Align::Start)
                    .spacing(20)
                    .push(Text::new(prompt.as_str())
                        .size(global.text_size("XLARGE")))
                    .push(row)
            }

            Question::MultiChoice {
                prompt,
                options,
                answer
            } => {
                let mut row = Row::new()
                    // .width(Length::Fill)
                    .spacing(40);
                for i in 0..options.len() {
                    let ind = index.clone();
                    row = row.push(Checkbox::new(
                        answer[i],
                        options[i].clone(),
                        move |_value| Message::UIEvent(
                            (0x01 + ind) as u16,
                            Value::Integer(i as i32)))
                        // .width(Length::Units(250))
                        .text_size(global.text_size("XLARGE"))
                        .size(global.text_size("LARGE")));
                }

                Column::new()
                    // .width(Length::Fill)
                    .align_items(Align::Start)
                    .spacing(20)
                    .push(Text::new(prompt.as_str())
                        .size(global.text_size("XLARGE")))
                    .push(row)
            }

            Question::ShortAnswer {
                prompt,
                answer,
                handles
            } => {
                let ind = index.clone();
                let [h_text_input] = handles;
                let e_text_input = TextInput::new(
                    h_text_input,
                    "Enter answer",
                    answer.as_str(),
                    move |value| Message::UIEvent(
                        (0x01 + ind) as u16,
                        Value::String(value)))
                    .size(global.text_size("XLARGE"))
                    .width(Length::Units(600));

                Column::new()
                    // .width(Length::Fill)
                    .align_items(Align::Start)
                    .spacing(20)
                    .push(Text::new(prompt.as_str())
                        .size(global.text_size("XLARGE")))
                    .push(e_text_input)
            }

            Question::Slider {
                prompt,
                answer,
                range,
                step,
                handles,
                ..
            } => {
                let ind = index.clone();
                let [h_slider] = handles;
                let e_slider = iced::Slider::new(
                    h_slider,
                    (*range).clone(),
                    *answer,
                    move |value| Message::UIEvent(
                        (0x01 + ind) as u16,
                        Value::Float(value)))
                    .step(*step)
                    .width(Length::Units(500));

                Column::new()
                    // .width(Length::Fill)
                    .align_items(Align::Start)
                    .spacing(20)
                    .push(Text::new(prompt.as_str())
                        .size(global.text_size("XLARGE")))
                    .push(Row::new()
                        .spacing(20)
                        .push(Text::new(range.start().to_string())
                            .size(global.text_size("LARGE")))
                        .push(e_slider)
                        .push(Text::new(range.end().to_string())
                            .size(global.text_size("LARGE")))
                    )
            }
        }
    }
}

pub mod run {
    use std::path::PathBuf;
    use std::sync::mpsc::TryRecvError;
    use rodio::OutputStreamHandle;
    use super::*;

    pub async fn instruction(id: ID, comm: Comm, mut timer: u16) -> Message {
        while timer > 0 {
            std::thread::sleep(Duration::from_secs(1));
            match comm.1.try_recv() {
                Ok(Message::Wrap) |
                Ok(Message::Interrupt) |
                Err(TryRecvError::Disconnected) => {
                    return Message::Null;
                },
                Err(TryRecvError::Empty) => (),
                Ok(msg) => panic!("Unexpected message received: {:?}", msg),
            }
            timer -= 1;
        }
        Message::ActionComplete(id)
    }

    pub async fn audio(id: ID, comm: Comm, source: PathBuf, use_trigger: bool, stream_handle: OutputStreamHandle) -> Message {
        let trigger = source.with_extension("trig.wav");
        let trigger = if use_trigger { Some(trigger.as_path()) } else { None };

        match play_audio(comm, source.as_path(), trigger, stream_handle) {
            Ok(()) => Message::ActionComplete(id),
            Err(()) => Message::Null,
        }
    }
}

mod default {
    use super::*;

    pub fn timer() -> u16 {
        3
    }

    pub fn slider_range() -> RangeInclusive<f32> {
        0.0..=100.0
    }

    pub fn slider_step() -> f32 {
        0.01
    }
}

mod serialize {
    use serde::ser::SerializeMap;
    use serde::Serializer;

    pub mod question {
        use super::*;

        pub fn single_choice<S: Serializer>(
            prompt: &str,
            options: &Vec<String>,
            answer: &Option<usize>,
            s: S,
        ) -> Result<S::Ok, S::Error> {
            let mut map = s.serialize_map(Some(3))?;
            map.serialize_entry("prompt", prompt)?;
            map.serialize_entry("options", options)?;
            if let Some(ans) = answer {
                map.serialize_entry("answer", options[*ans].as_str())?;
            } else {
                map.serialize_entry("answer", "~")?;
            }
            map.end()
        }

        pub fn multi_choice<S: Serializer>(
            prompt: &str,
            options: &Vec<String>,
            answer: &Vec<bool>,
            s: S
        ) -> Result<S::Ok, S::Error> {
            let mut map = s.serialize_map(Some(3))?;
            let answer: Vec<_> = options.iter()
                .enumerate()
                .filter(|(i, _)| answer[*i])
                .map(|(_, o)| o.clone())
                .collect();
            map.serialize_entry("prompt", prompt)?;
            map.serialize_entry("options", options)?;
            map.serialize_entry("answer", &answer)?;
            map.end()
        }
    }
}
