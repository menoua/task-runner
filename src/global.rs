use std::collections::HashSet;
use std::fmt;
use iced::{Align, HorizontalAlignment};
use serde::{Serialize, Deserialize, de};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Global {
    #[serde(default="default::window_size", deserialize_with="deserialize::window_size")]
    window_size: (u32, u32),
    #[serde(default="default::min_window_size", deserialize_with="deserialize::window_size")]
    min_window_size: (u32, u32),
    #[serde(default="default::content_size", deserialize_with="deserialize::content_size")]
    content_size: (IntOrFloat, IntOrFloat),
    #[serde(default="default::resizable")]
    resizable: bool,
    #[serde(default="default::font_scale")]
    font_scale: f32,
    #[serde(default="default::text_alignment")]
    text_alignment: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum IntOrFloat {
    Integer(u32),
    Float(f32),
}

impl Default for IntOrFloat {
    fn default() -> Self {
        IntOrFloat::Float(1.0)
    }
}

mod deserialize {
    use super::*;

    pub fn window_size<'de, D>(deserializer: D) -> Result<(u32, u32), D::Error> where
        D: de::Deserializer<'de>
    {
        struct WindowSizeVisitor;

        impl<'de> de::Visitor<'de> for WindowSizeVisitor {
            type Value = (u32, u32);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string like 1024 x 768")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                let v = v.to_string();
                let (x, y) = v.split_once('x').unwrap();
                Ok((x.trim().parse().unwrap(), y.trim().parse().unwrap()))
            }
        }

        deserializer.deserialize_any(WindowSizeVisitor)
    }

    pub fn content_size<'de, D>(deserializer: D) -> Result<(IntOrFloat, IntOrFloat), D::Error> where
        D: de::Deserializer<'de>
    {
        struct ContentSizeVisitor;

        impl<'de> de::Visitor<'de> for ContentSizeVisitor {
            type Value = (IntOrFloat, IntOrFloat);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string like 1024 x 768, or 0.8 x 0.8")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                let v = v.to_string();
                let (x, y) = v.split_once('x').unwrap();

                let x = match x.trim().parse::<u32>() {
                    Ok(i) => IntOrFloat::Integer(i),
                    Err(_) => IntOrFloat::Float(x.trim().parse::<f32>()
                        .expect("Content width should be a valid positive number")),
                };
                let y = match y.trim().parse::<u32>() {
                    Ok(i) => IntOrFloat::Integer(i),
                    Err(_) => IntOrFloat::Float(y.trim().parse::<f32>()
                        .expect("Content height should be a valid positive number")),
                };

                Ok((x, y))
            }
        }

        deserializer.deserialize_any(ContentSizeVisitor)
    }
}

mod default {
    use crate::global::IntOrFloat;

    pub fn window_size() -> (u32, u32) {
        (900, 780)
    }

    pub fn min_window_size() -> (u32, u32) {
        (600, 600)
    }

    pub fn content_size() -> (IntOrFloat, IntOrFloat) {
        (IntOrFloat::Float(0.8), IntOrFloat::Float(0.8))
    }

    pub fn resizable() -> bool {
        true
    }

    pub fn font_scale() -> f32 {
        1.0
    }

    pub fn text_alignment() -> String {
        "Center".to_string()
    }
}

impl Global {
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    pub fn min_window_size(&self) -> Option<(u32, u32)> {
        Some(self.min_window_size)
    }

    pub fn content_size(&self) -> (IntOrFloat, IntOrFloat) {
        self.content_size
    }

    pub fn resizable(&self) -> bool {
        self.resizable
    }

    pub fn font_scale(&self) -> f32 {
        self.font_scale
    }

    pub fn alignment(&self) -> Align {
        match self.text_alignment.to_uppercase().as_str() {
            "START" | "LEFT" => Align::Start,
            "CENTER" => Align::Center,
            "END" | "RIGHT" => Align::End,
            _ => panic!("Invalid text alignment value")
        }
    }

    pub fn horizontal_alignment(&self) -> HorizontalAlignment {
        match self.text_alignment.to_uppercase().as_str() {
            "START" | "LEFT" => HorizontalAlignment::Left,
            "CENTER" => HorizontalAlignment::Center,
            "END" | "RIGHT" => HorizontalAlignment::Right,
            _ => panic!("Invalid text alignment value")
        }
    }

    pub fn text_size(&self, scale: &str) -> u16 {
        let size = match scale.to_uppercase().as_str() {
            "TINY" => 16,
            "SMALL" => 20,
            "NORMAL" => 24,
            "LARGE" => 28,
            "XLARGE" => 32,
            "XXLARGE" => 36,
            _ => panic!("Unknown font scale {}", scale),
        };
        (self.font_scale * size as f32).round() as u16
    }

    pub fn verify(&self) {
        match self.content_size.0 {
            IntOrFloat::Integer(i) if (i == 0 || i > self.window_size.0) => {
                panic!("Content width should be positive and less than or equal to window width");
            }
            IntOrFloat::Float(f) if (f <= 0.01 || f > 0.99) => {
                panic!("Fractional content width should be between 0.01 and 0.99 inclusive");
            }
            _ => (),
        }
        match self.content_size.1 {
            IntOrFloat::Integer(i) if (i == 0 || i > self.window_size.1) => {
                panic!("Content height should be positive and less than or equal to window height");
            }
            IntOrFloat::Float(f) if (f <= 0.01 || f > 0.99) => {
                panic!("Fractional content height should be between 0.01 and 0.99 inclusive");
            }
            _ => (),
        }

        if self.font_scale < 0.5 || self.font_scale > 3.0 {
            panic!("Font scale should be between 0.5 and 3.0");
        }

        let possible_alignments = HashSet::from([
            "START", "LEFT", "CENTER", "END", "RIGHT"
        ]);
        if !possible_alignments.contains(self.text_alignment.to_uppercase().as_str()) {
            panic!("Text alignment should be one of: {:?}", possible_alignments);
        }
    }
}
