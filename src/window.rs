use std::fmt;
use serde::{Serialize, Deserialize, de};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    #[serde(default="default::size", deserialize_with="pair_from_x")]
    size: (u32, u32),
    #[serde(default)]
    resizable: bool,
    #[serde(default="default::font_scale")]
    font_scale: f32,
}

fn pair_from_x<'de, D>(deserializer: D) -> Result<(u32, u32), D::Error> where
    D: de::Deserializer<'de>
{
    struct WindowSizeVisitor;

    impl<'de> de::Visitor<'de> for WindowSizeVisitor {
        type Value = (u32, u32);

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
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

    // use our visitor to deserialize a window size
    deserializer.deserialize_any(WindowSizeVisitor)
}

mod default {
    pub fn size() -> (u32, u32) {
        (900, 780)
    }

    pub fn font_scale() -> f32 {
        1.0
    }
}

impl Window {
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn resizable(&self) -> bool {
        self.resizable
    }

    pub fn font_scale(&self) -> f32 {
        self.font_scale
    }
}
