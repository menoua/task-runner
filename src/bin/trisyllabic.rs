use iced::{window, Application, Settings};
use serde::Serialize;

use neurotask::block::{AudioBlock, Block, Communication};
use neurotask::config::{AudioConfig, Config, StdConfig, StdConfigItem};
use neurotask::error::Error;
use neurotask::question::{StdAnswer, StdQuestion};
use neurotask::task::{Builder, Task};

#[derive(Debug, Clone, Serialize)]
pub struct TrisyllabicBlock {
    id: String,
}

impl Block for TrisyllabicBlock {
    type Config = StdConfig;
    type Question = StdQuestion;
    type Answer = StdAnswer;

    fn id(&self) -> String {
        match self.id.as_str() {
            "1" | "2" | "3" => self.id.clone(),
            _ => panic!("Invalid block `{}`", self.id),
        }
    }

    fn title(&self) -> String {
        String::from(match self.id.as_str() {
            "1" => "1st Block",
            "2" => "2nd Block",
            "3" => "3rd Block",
            _ => panic!("Invalid block `{}`", self.id),
        })
    }

    fn description(&self) -> String {
        String::from("Press the button every time you hear the word OBJECTIVE.")
    }

    fn run(id: String, config: Self::Config, comm: Communication) -> Result<(), Error> {
        let audio_cfg = match config.get("audio_cfg") {
            StdConfigItem::AudioConfig(cfg) => cfg,
            _ => panic!("Invalid audio configuration"),
        };

        let use_trigger = match audio_cfg {
            AudioConfig::MonoAudioAndTrigger => true,
            AudioConfig::StereoAudio => false,
        };

        <Self as AudioBlock>::run(id, use_trigger, comm)
    }

    fn questionnaire(
        id: &str,
        _config: &StdConfig,
        // _comm: Communication
    ) -> Vec<StdQuestion> {
        vec![
            StdQuestion::single_choice(
                &match id {
                    i @ ("1" | "2" | "3") => format!("Q{}", i),
                    _ => panic!("Invalid block id `{}`", id),
                },
                "Which of the following words were NOT spoken so far?",
                &match id {
                    "1" => vec!["Salary", "Limited", "Governor"],
                    "2" => vec!["Visitor", "Prominent", "Relative"],
                    "3" => vec!["Pacific", "Traveler", "Reception"],
                    _ => panic!("Invalid block id `{}`", id),
                },
            ),
            StdQuestion::multi_choice(
                &match id {
                    i @ ("1" | "2" | "3") => format!("Q{}", i),
                    _ => panic!("Invalid block id `{}`", id),
                },
                "Which of the following words were NOT spoken so far?",
                &match id {
                    "1" => vec!["Salary", "Limited", "Governor"],
                    "2" => vec!["Visitor", "Prominent", "Relative"],
                    "3" => vec!["Pacific", "Traveler", "Reception"],
                    _ => panic!("Invalid block id `{}`", id),
                },
            ),
        ]
    }
}

impl AudioBlock for TrisyllabicBlock {
    fn audio_src(id: &str) -> String {
        match id {
            i @ ("1" | "2" | "3") => format!("block{}.wav", i),
            _ => panic!("Invalid block id `{}`", id),
        }
    }
}

fn init() -> Builder<TrisyllabicBlock> {
    fn f() -> Task<TrisyllabicBlock> {
        const DESCRIPTION: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/text/description.txt"
        ));

        Task::new()
            .set_title("Trisyllabic")
            .set_version("1.4")
            .set_config(StdConfig::default())
            .set_blocks(vec![
                TrisyllabicBlock {
                    id: String::from("1"),
                },
                TrisyllabicBlock {
                    id: String::from("2"),
                },
                TrisyllabicBlock {
                    id: String::from("3"),
                },
            ])
            .set_description(DESCRIPTION)
    }

    Some(Box::new(f))
}

pub fn main() -> iced::Result {
    Task::run(Settings {
        flags: init(),
        window: window::Settings {
            size: (1024, 768),
            resizable: false,
            ..Default::default()
        },
        ..Default::default()
    })
}
