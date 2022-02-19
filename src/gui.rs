use std::fmt;
use iced::{Align, HorizontalAlignment};
use serde::{Serialize, Deserialize, de};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GUI {
    #[serde(default="default::window_size", deserialize_with="deserialize::window_size")]
    window_size: (u32, u32),
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
                formatter.write_str("a string like 1024 x 768")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
            {
                let v = v.to_string();
                let (x, y) = v.split_once('x').unwrap();

                let x = match x.trim().parse::<u32>() {
                    Ok(i) => IntOrFloat::Integer(i),
                    Err(_) => IntOrFloat::Float(x.trim().parse::<f32>().unwrap()),
                };
                let y = match y.trim().parse::<u32>() {
                    Ok(i) => IntOrFloat::Integer(i),
                    Err(_) => IntOrFloat::Float(y.trim().parse::<f32>().unwrap()),
                };

                if let IntOrFloat::Float(f) = x {
                    if f < 0.0 || f > 1.0 {
                        panic!("x and y should either b integers denoting pixels or a float between 0 and 1")
                    }
                }
                if let IntOrFloat::Float(f) = y {
                    if f < 0.0 || f > 1.0 {
                        panic!("x and y should either b integers denoting pixels or a float between 0 and 1")
                    }
                }

                Ok((x, y))
            }
        }

        deserializer.deserialize_any(ContentSizeVisitor)
    }
}

mod default {
    use crate::gui::IntOrFloat;

    pub fn window_size() -> (u32, u32) {
        (900, 780)
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

impl GUI {
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
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
        match self.text_alignment.to_lowercase().as_str() {
            "start" | "left" => Align::Start,
            "center" => Align::Center,
            "end" | "right" => Align::End,
            _ => panic!("Invalid text alignment value")
        }
    }

    pub fn horizontal_alignment(&self) -> HorizontalAlignment {
        match self.text_alignment.to_lowercase().as_str() {
            "start" | "left" => HorizontalAlignment::Left,
            "center" => HorizontalAlignment::Center,
            "end" | "right" => HorizontalAlignment::Right,
            _ => panic!("Invalid text alignment value")
        }
    }
}
