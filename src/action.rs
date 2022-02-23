use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::ops::RangeInclusive;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use iced::{image, Column, Length, Text, Align, button, Checkbox, TextInput, text_input, Space, Container, slider, Row};
use iced_futures::Command;
use iced_native::Image;

use crate::comm::{Comm, Message, Receiver, Sender, Value};
use crate::sound::play_audio;
use crate::util::{timestamp, async_write_to_file, resource, template, output};
use crate::global::Global;
use crate::style::button;

use Question::*;

pub type ID = String;
pub const MAX_DEPTH: u16 = 3;

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
    #[serde(skip)]
    dependents: HashSet<ID>,
    #[serde(skip)]
    successors: HashSet<ID>,
    #[serde(skip)]
    expired: Option<bool>,
    #[serde(skip)]
    log_prefix: String,
    #[serde(skip)]
    comm: Option<Sender>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Action {
    Nothing {
        #[serde(default, flatten)]
        info: Info,
    },
    Instruction {
        prompt: String,
        #[serde(default="default::timer")]
        timer: u16,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        handle: Option<button::State>,
    },
    Selection {
        prompt: String,
        options: Vec<String>,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip_deserializing)]
        choice: Option<usize>,
        #[serde(skip)]
        handles: Vec<button::State>,
    },
    Audio {
        source: String,
        #[serde(default, flatten)]
        info: Info,
    },
    Image {
        source: String,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        handle: Option<image::Handle>,
    },
    Question {
        list: Vec<Question>,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        handle: button::State,
    },
    // AudioSequence { .. },
    // ImageSequence { .. },
    // QuestionSequence { .. },
    Template {
        source: String,
        #[serde(default)]
        params: HashMap<String, String>,
        #[serde(default, flatten)]
        info: Info,
        #[serde(skip)]
        actions: Vec<Action>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
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
        handle: text_input::State,
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
        handle: slider::State,
    },
}

impl Question {
    pub fn init(&mut self) {
        match self {
            MultiChoice { options, answer, .. } => {
                *answer = vec![false; options.len()];
            }
            Slider { answer, range, .. } => {
                *answer = *range.start();
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
        position: usize,
        last_action: &Option<ID>,
        depth: u16,
        task_dir: &Path
    ) -> Result<(), String> {
        if depth > MAX_DEPTH {
            return Err(format!("Maximum allowed template depth reached: {}.", MAX_DEPTH));
        }
        let info = self.info_mut();
        if info.id.is_empty() {
            info.id = position.to_string();
        } else if !info.id.chars().all(|c| c.is_ascii_alphanumeric() || "_-".contains(c)) {
            return Err("Only alphanumeric (a-z|A-Z|0-9), '-', and '_' are allowed in actions IDs.".to_string());
        } else if info.id.chars().all(char::is_numeric) {
            return Err("Custom action ID cannot be digits only.".to_string());
        } else if info.id == "entry" || info.id == "exit" {
            return Err("`entry` and `exit` are reserved action IDs.".to_string());
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
            let file = resource(task_dir, file)?;
            info.background_image = Some(image::Handle::from_path(file));
        }
        if let Some(0) = info.timeout {
            info.expired = Some(true);
        }

        match self {
            Action::Nothing { info, .. } => {
                if info.timeout.is_none() {
                    info.timeout = Some(0);
                }
            }
            Action::Instruction { timer, handle, .. } => {
                *handle = if *timer == 0 {
                    Some(button::State::new())
                } else {
                    None
                };
            }
            Action::Selection { options, handles, .. } => {
                *handles = vec![button::State::new(); options.len()];
            }
            Action::Audio { .. } => {
                ()
            }
            Action::Image { handle, source, .. } => {
                let source = resource(task_dir, source)?;
                *handle = Some(image::Handle::from_path(source));
            }
            Action::Question { list, .. } => {
                for quest in list {
                    quest.init();
                }
            }
            Action::Template {
                source,
                params,
                actions,
                info,
                ..
            } => {
                let file = template(task_dir, source)?;
                let mut file = File::open(file)
                    .or(Err(format!("Failed to open template file: {:?}", source)))?;

                let mut content = String::new();
                file.read_to_string(&mut content)
                    .or(Err(format!("Invalid UTF-8 text in template file: {:?}", source)))?;

                for (k, v) in params {
                    let k = format!("{{{{{}}}}}", k);
                    if !content.contains(&k) {
                        return Err(format!("Invalid template parameter \"{}\" specified for template file: {:?}", k, source));
                    }
                    content = content.replace(&k, v);
                }
                if content.contains("{{") {
                    return Err("All parameters in a template should have specified values".to_string());
                }

                *actions = serde_yaml::from_str(&content).or_else(|e|
                    Err(format!("Failed to parse template \"{}\" at line {}: {}",
                                source, e.location().unwrap().line(), e)))?;

                let mut last_action = None;
                let mut ids = HashSet::new();
                for (i, action) in actions.iter_mut().enumerate() {
                    action.init(i+1, &last_action, 1+depth, task_dir)?;
                    last_action = Some(action.id());

                    let id = action.id();
                    if ids.contains(&id) {
                        return Err(format!("Action ID `{}` used more than once in template: {}", id, source));
                    } else {
                        ids.insert(id);
                    }
                }

                let mut i: usize = 0;
                while i < actions.len() {
                    if matches!(actions[i], Action::Template { .. }) {
                        if let Action::Template { actions: inners, .. } = actions[i].clone() {
                            actions.remove(i);
                            for inner in inners.into_iter() {
                                actions.insert(i, inner);
                                i += 1;
                            }
                        }
                    } else {
                        i += 1;
                    }
                }

                for action in actions.iter_mut() {
                    let inner_info = action.info_mut();
                    inner_info.id = format!("{}~{}", info.id, inner_info.id);
                    if let Some(after) = &mut inner_info.after {
                        *after = after.iter().map(|x| format!("{}~{}", info.id, x)).collect();
                        if let Some(ids) = &info.after {
                            after.extend(ids.clone());
                        }
                    } else {
                        info.after = info.after.clone();
                    }
                    if let Some(id) = &info.with {
                        info.with = Some(format!("{}~{}", info.id, id));
                    } else {
                        info.with = info.with.clone();
                    }
                }

                flow::add_gates(actions, info.after.clone(), info.with.clone())?;

                let len = actions.len();
                actions[0].set_id(&format!("{}~entry", info.id));
                actions[len-1].set_id(&format!("{}~exit", info.id));
            }
        }

        Ok(())
    }

    pub fn id(&self) -> ID {
        self.info().id.clone()
    }

    pub fn set_id(&mut self, id: &ID) {
        self.info_mut().id = id.clone();
    }

    pub fn is(&self, id: &str) -> bool {
        self.id() == id
    }

    pub fn info(&self) -> &Info {
        match self {
            Action::Nothing { info, .. } |
            Action::Instruction { info, .. } |
            Action::Selection { info, .. } |
            Action::Audio { info, .. } |
            Action::Image { info, .. } |
            Action::Question { info, .. } |
            Action::Template { info, .. } => info
        }
    }

    pub fn info_mut(&mut self) -> &mut Info {
        match self {
            Action::Nothing { info, .. } |
            Action::Instruction { info, .. } |
            Action::Selection { info, .. } |
            Action::Audio { info, .. } |
            Action::Image { info, .. } |
            Action::Question { info, .. } |
            Action::Template { info, .. } => info
        }
    }

    pub fn with(&self) -> Option<ID> {
        self.info().with.clone()
    }

    pub fn after(&self) -> HashSet<ID> {
        if let Some(ids) = &self.info().after {
            ids.clone()
        } else {
            HashSet::new()
        }
    }

    pub fn dependents(&self) -> &HashSet<ID> {
        &self.info().dependents
    }

    pub fn add_dependent(&mut self, id: ID) {
        self.info_mut().dependents.insert(id);
    }

    pub fn expire(&mut self) {
        self.info_mut().expired = Some(true);
    }

    pub fn successors(&self) -> &HashSet<ID> {
        &self.info().successors
    }

    pub fn add_successor(&mut self, id: ID) {
        self.info_mut().successors.insert(id);
    }

    pub fn satisfy(&mut self, id: &ID) -> bool {
        self.info_mut().after.as_mut().unwrap().remove(id);
        self.is_ready().unwrap()
    }

    pub fn verify(&mut self, id_list: &HashSet<ID>) -> Result<(), String> {
        let info = self.info_mut();
        match info {
            Info { after: Some(ids), .. } if ids.contains(&info.id) => {
                Err(format!("Action cannot be a successor of itself: {}", info.id))
            }
            Info { with: Some(id), .. } if *id == info.id => {
                Err(format!("Action cannot be a dependent of itself: {}", info.id))
            }
            Info { after, with, .. } => {
                // Relink template successors to exit point
                if let Some(after) = after {
                    *after = after.iter()
                        .map(|id| {
                            if id_list.contains(id) {
                                Ok(id.to_owned())
                            } else if id_list.contains(&format!("{}~exit", id)) {
                                Ok(format!("{}~exit", id))
                            } else {
                                Err(format!("Invalid action ID: {}", id))
                            }
                        })
                        .collect::<Result<HashSet<ID>, String>>()?;
                }
                // Relink template dependents to entry/exit points
                if let Some(id) = with {
                    if id_list.contains(id) {
                        ()
                    } else if let Some(after) = after {
                        after.insert(format!("{}~entry", id));
                        *id = format!("{}~exit", id);
                    } else {
                        *after = Some(HashSet::from([format!("{}~entry", id)]));
                        *id = format!("{}~exit", id);
                    }
                }
                Ok(())
            }
        }
    }

    pub fn is_ready(&self) -> Option<bool> {
        if let Some(ids) = &self.info().after {
            Some(ids.is_empty())
        } else {
            None
        }
    }

    pub fn is_expired(&self) -> Option<bool> {
        self.info().expired
    }

    pub fn has_view(&self) -> bool {
        match self {
            Action::Nothing { .. } |
            Action::Audio { .. } => false,

            Action::Instruction { .. } |
            Action::Selection { .. } |
            Action::Image { .. } |
            Action::Question { .. } => true,

            Action::Template { .. } => todo!(),
        }
    }

    pub fn has_background(&self) -> bool {
        self.info().background.is_some()
    }

    pub fn captures_keystrokes(&self) -> bool {
        self.info().monitor_kb
    }

    pub fn run(&mut self, writer: Sender, log_dir: &str, global: &Global) -> Command<Message> {
        self.info_mut().log_prefix = output(log_dir, &self.id());

        let mut commands = vec![];
        if let Some(t) = self.info().timeout {
            let id = self.id();
            commands.push(Command::perform(async move {
                std::thread::sleep(Duration::from_millis(t as u64));
                Message::ActionComplete(id)
            }, |msg| msg));
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
                let source = resource(Path::new(global.dir()), source).unwrap();
                let use_trigger = global.config().use_trigger();
                let stream_handle = global.io().audio_stream();

                let source = source.clone();
                let rx = self.new_comm_link();
                commands.push(Command::perform(
                    run::audio(self.id(), (writer, rx), source, use_trigger, stream_handle),
                    |msg| msg));
            }
            Action::Nothing { .. } |
            Action::Selection { .. } |
            Action::Image { .. } |
            Action::Question { .. } |
            Action::Template { .. } => {}
        }

        Command::batch(commands)
    }

    pub fn view(&mut self, global: &Global) -> Column<Message> {
        let id = self.id();
        match self {
            Action::Nothing { .. } => {
                Column::new()
            }
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
            Action::Selection { prompt, options, handles, .. } => {
                let mut rows = Column::new()
                    .spacing(40)
                    .align_items(Align::Center);
                let mut controls = Row::new()
                    .spacing(60);
                for (i, handle) in handles.iter_mut().enumerate() {
                    if i > 0 && i % 3 == 0 {
                        rows = rows.push(controls);
                        controls = Row::new()
                            .spacing(60);
                    }
                    controls = controls.push(button(
                        handle,
                        &options[i],
                        global.text_size("XLARGE"))
                        .on_press(Message::UIEvent(0x01, Value::Integer(1+i as i32)))
                        .width(Length::Units(200)));
                }
                rows = rows.push(controls);

                Column::new()
                    // .width(Length::Fill)
                    .spacing(40)
                    .align_items(Align::Center)
                    .push(Text::new(prompt.as_str())
                        .size(global.text_size("XLARGE")))
                    .push(rows)
                    .into()
            }
            Action::Audio { .. } => {
                Column::new()
            }
            Action::Image { handle, .. } => {
                let image = handle.as_ref().unwrap().clone();
                let image = Image::new(image);

                Column::new()
                    .push(Container::new(image)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x()
                        .center_y())
                    .width(Length::Fill)
                    .height(Length::Fill)
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
            Action::Template { .. } => {
                Column::new()
                    .push(Text::new("This shouldn't have happened!")
                        .size(global.text_size("XLARGE")))
            }
        }
    }

    pub fn update(&mut self, message: Message, _global: &Global) -> Command<Message> {
        if let Message::KeyPress(key_code) = message {
            self.info_mut().keystrokes.push(format!("{}  {:?}", timestamp(), key_code));
            return Command::none();
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
            Action::Selection { choice, .. } => {
                match message {
                    Message::UIEvent(0x01, Value::Integer(i)) => {
                        *choice = Some(i as usize);
                        let id = self.id();
                        Command::perform(
                            async move { id },
                            |id| Message::ActionComplete(id))
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
            _ => {
                panic!("{:?}", message)
            }
        }
    }

    pub fn background(&mut self) -> Column<Message> {
        let image = self.info_mut().background_image.as_ref().unwrap().clone();
        let image = Image::new(image);

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
        let info = self.info();
        if info.monitor_kb {
            async_write_to_file(
                format!("{}.keypress", info.log_prefix),
                info.keystrokes.clone(),
                "Failed to write key presses to output file");
        }
        if let Some(comm) = &info.comm {
            comm.send(Message::Wrap).ok();
        }

        match self {
            Action::Selection { info, choice, .. } => {
                async_write_to_file(
                    format!("{}.choice", info.log_prefix),
                    choice.clone(),
                    "Failed to write selection choice to output file");
            }
            Action::Question { info, list, .. } => {
                async_write_to_file(
                    format!("{}.response", info.log_prefix),
                    list.clone(),
                    "Failed to write question responses to output file");
            }
            _ => (),
        }
    }

    pub fn new_comm_link(&mut self) -> Receiver {
        let (tx, rx) = mpsc::channel();
        self.info_mut().comm = Some(tx);
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
                handle
            } => {
                let ind = index.clone();
                let e_text_input = TextInput::new(
                    handle,
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
                handle,
                ..
            } => {
                let ind = index.clone();
                let e_slider = iced::Slider::new(
                    handle,
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
            std::thread::sleep(Duration::from_millis(1));
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
        3_000
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

pub mod flow {
    use super::*;

    pub fn add_gates(
        actions: &mut Vec<Action>,
        after: Option<HashSet<ID>>,
        with: Option<ID>
    ) -> Result<(), String> {
        let entry = Action::Nothing {
            info: Info {
                id: "entry".to_string(),
                with: with.clone(),
                after: after.clone(),
                monitor_kb: false,
                keystrokes: vec![],
                background: None,
                background_image: None,
                timeout: Some(0),
                dependents: Default::default(),
                successors: Default::default(),
                expired: Some(true),
                log_prefix: "".to_string(),
                comm: None
            }
        };

        let mut finalists: HashSet<ID> = actions.iter().map(Action::id).collect();

        for action in actions.iter_mut() {
            let inner_info = action.info_mut();
            if let Some(after) = &mut inner_info.after {
                for x in after.iter() {
                    finalists.remove(x);
                }
                after.insert("entry".to_string());
            }
            if inner_info.with.is_none() {
                inner_info.with = with.clone();
            }
        }

        let exit = Action::Nothing {
            info: Info {
                id: "exit".to_string(),
                with: with.clone(),
                after: Some(finalists),
                monitor_kb: false,
                keystrokes: vec![],
                background: None,
                background_image: None,
                timeout: Some(0),
                dependents: Default::default(),
                successors: Default::default(),
                expired: Some(true),
                log_prefix: "".to_string(),
                comm: None
            }
        };

        actions.insert(0, entry);
        actions.push(exit);
        Ok(())
    }
}
