use serde::Serialize;
use std::fmt::Debug;

pub use standard::{AudioConfig, InputConfig};
pub use standard::{StdConfig, StdConfigItem};

pub trait Config: Debug + Default + Clone + Serialize + Send {
    type Item: Debug + Clone + Send + PartialEq;

    fn keys() -> Vec<&'static str>;
    fn values(key: &str) -> Vec<(&'static str, Self::Item)>;
    fn description(key: &str) -> &'static str;

    fn get(&self, key: &str) -> Self::Item;
    fn update(&mut self, key: &str, value: Self::Item);
}

mod standard {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum InputConfig {
        External,
        Keyboard,
    }

    impl Default for InputConfig {
        fn default() -> Self {
            InputConfig::External
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum AudioConfig {
        MonoAudioAndTrigger,
        StereoAudio,
    }

    impl Default for AudioConfig {
        fn default() -> Self {
            AudioConfig::MonoAudioAndTrigger
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum StdConfigItem {
        InputConfig(InputConfig),
        AudioConfig(AudioConfig),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct StdConfig {
        input_cfg: InputConfig,
        audio_cfg: AudioConfig,
    }

    impl Default for StdConfig {
        fn default() -> Self {
            StdConfig {
                input_cfg: InputConfig::default(),
                audio_cfg: AudioConfig::default(),
            }
        }
    }

    impl Config for StdConfig {
        type Item = StdConfigItem;

        fn keys() -> Vec<&'static str> {
            vec!["input_cfg", "audio_cfg"]
        }

        fn values(key: &str) -> Vec<(&'static str, Self::Item)> {
            match key {
                "input_cfg" => vec![
                    (
                        "External device",
                        StdConfigItem::InputConfig(InputConfig::External),
                    ),
                    (
                        "System keyboard",
                        StdConfigItem::InputConfig(InputConfig::Keyboard),
                    ),
                ],

                "audio_cfg" => vec![
                    (
                        "L: Audio / R: Trigger",
                        StdConfigItem::AudioConfig(AudioConfig::MonoAudioAndTrigger),
                    ),
                    (
                        "Stereo audio",
                        StdConfigItem::AudioConfig(AudioConfig::StereoAudio),
                    ),
                ],

                _ => panic!("`{}` not found in configuration", key),
            }
        }

        fn description(key: &str) -> &'static str {
            match key {
                "input_cfg" => "Input (reaction) device configuration",
                "audio_cfg" => "Output audio channel configuration",
                _ => panic!("`{}` not found in configuration", key),
            }
        }

        fn get(&self, key: &str) -> Self::Item {
            match key {
                "input_cfg" => StdConfigItem::InputConfig(self.input_cfg.clone()),
                "audio_cfg" => StdConfigItem::AudioConfig(self.audio_cfg.clone()),
                _ => panic!("`{}` not found in configuration", key),
            }
        }

        fn update(&mut self, key: &str, value: Self::Item) {
            match key {
                "input_cfg" => match value {
                    StdConfigItem::InputConfig(cfg) => {
                        self.input_cfg = cfg;
                    }
                    _ => panic!("bad value for config `{}`", key),
                },

                "audio_cfg" => match value {
                    StdConfigItem::AudioConfig(cfg) => {
                        self.audio_cfg = cfg;
                    }
                    _ => panic!("bad value for config `{}`", key),
                },

                _ => panic!("`{}` not found in configuration", key),
            }
        }
    }
}
