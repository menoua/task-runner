use iced::{Column, Length, Row, Text, button, Radio};
use iced_native::Space;
use serde::{Serialize, Deserialize};

use crate::comm::{Code, Message, Value};
use crate::global::Global;
use crate::style::{self, button};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Config {
    audio: (AudioConfig, bool),
    #[serde(skip)]
    handles: [button::State; 3],
}

impl Config {
    pub fn is_static(&self) -> bool {
        self.audio.1
    }

    pub fn view(&mut self, global: &Global) -> Column<Message> {
        let mut content = Column::new()
            .width(Length::Fill)
            .spacing(60)
            .align_items(global.alignment())
            .push(Text::new("Configuration")
                .size(global.text_size("XLARGE"))
                .horizontal_alignment(global.horizontal_alignment()));

        if !self.audio.1 {
            content = content.push(self.audio.0.view(global));
        }
        content = content.push(Space::with_height(Length::Fill));

        let [h_cancel, h_revert, h_start] = &mut self.handles;
        let e_cancel = button(
            h_cancel,
            "Cancel",
            global.text_size("LARGE"))
            .on_press(Message::UIEvent(0x01, Value::Null))
            .style(style::Button::Secondary)
            .width(Length::Units(200))
            .padding(15);
        let e_revert = button(
            h_revert,
            "Revert",
            global.text_size("LARGE"))
            .on_press(Message::UIEvent(0x02, Value::Null))
            .style(style::Button::Destructive)
            .width(Length::Units(200))
            .padding(15);
        let e_start = button(
            h_start,
            "Start!",
            global.text_size("LARGE"))
            .on_press(Message::UIEvent(0x03, Value::Null))
            .style(style::Button::Primary)
            .width(Length::Units(200))
            .padding(15);

        content.push(Row::new()
            .push(e_cancel)
            .push(Space::with_width(Length::Fill))
            .push(e_revert)
            .push(Space::with_width(Length::Fill))
            .push(e_start))
    }

    pub fn reset(&mut self) {
        self.audio.0 = AudioConfig::default();
    }

    pub fn update(&mut self, code: Code, value: Value) {
        match (code, value) {
            (0x04, Value::Integer(i)) => {
                self.audio.0 = match i {
                    1 => AudioConfig::MonoAndTrigger,
                    2 => AudioConfig::Stereo,
                    _ => panic!("Invalid value for audio config")
                };
            }

            _ => panic!("Invalid configuration code or value type")
        }
    }

    pub fn use_trigger(&self) -> bool {
        matches!(self.audio.0, AudioConfig::MonoAndTrigger)
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone)]
pub enum AudioConfig {
    MonoAndTrigger,
    Stereo,
}

impl Default for AudioConfig {
    fn default() -> Self { AudioConfig::MonoAndTrigger }
}

impl AudioConfig {
    pub fn view(&mut self, global: &Global) -> Column<Message> {
        let e_mono_t = Radio::new(
            AudioConfig::MonoAndTrigger,
            "L: Audio / R: Trigger",
            Some(self.clone()),
            |_| Message::UIEvent(0x04, Value::Integer(1)))
            .text_size(global.text_size("LARGE"));
        let e_stereo = Radio::new(
            AudioConfig::Stereo,
            "Stereo audio",
            Some(self.clone()),
            |_| Message::UIEvent(0x04, Value::Integer(2)))
            .text_size(global.text_size("LARGE"));

        Column::new()
            .align_items(global.alignment())
            .spacing(25)
            .push(Text::new("Output audio channel configuration")
                      .size(global.text_size("LARGE")))
            .push(Row::new()
                .spacing(40)
                .push(e_mono_t)
                // .push(Space::with_width(Length::Fill))
                .push(e_stereo))
    }
}

impl From<String> for AudioConfig {
    fn from(value: String) -> Self {
        match value.as_str() {
            "MonoAndTrigger" => AudioConfig::MonoAndTrigger,
            "Stereo" => AudioConfig::Stereo,
            _ => panic!("Unexpected value"),
        }
    }
}

impl Into<String> for AudioConfig {
    fn into(self) -> String {
        String::from(match self {
            AudioConfig::MonoAndTrigger => "MonoAndTrigger",
            AudioConfig::Stereo => "Stereo",
        })
    }
}
