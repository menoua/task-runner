use iced::{button, HorizontalAlignment, Text, VerticalAlignment};

pub use style::Button;

pub fn button<'a, T: Clone>(
    state: &'a mut button::State,
    text: &str,
    text_size: u16,
) -> iced::Button<'a, T> {
    let label = Text::new(text)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .size(text_size);

    iced::Button::new(state, label)
        .padding(10)
        .style(style::Button::Primary)
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Primary,
        Secondary,
        Destructive,
        Inactive,
        Active,
        Todo,
        Done,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb(0.11, 0.42, 0.87),
                    Button::Secondary => Color::from_rgb(0.5, 0.5, 0.5),
                    Button::Destructive => Color::from_rgb(0.8, 0.2, 0.2),
                    Button::Inactive => Color::WHITE,
                    Button::Active => Color::from_rgb(1.0, 0.9, 0.0),
                    Button::Todo => Color::WHITE,
                    Button::Done => Color::from_rgb(0.15, 0.76, 0.51),
                })),
                border_color: match self {
                    Button::Inactive => Color::from_rgb(1.0, 0.9, 0.0),
                    Button::Todo => Color::from_rgb(0.15, 0.76, 0.51),
                    _ => Color::TRANSPARENT,
                },
                border_width: match self {
                    Button::Inactive | Button::Todo => 2.0,
                    _ => 0.0,
                },
                border_radius: 16.0,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: match self {
                    Button::Inactive | Button::Active => Color::BLACK,
                    Button::Todo | Button::Done => Color::BLACK,
                    _ => Color::from_rgb8(0xEE, 0xEE, 0xEE),
                },
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                border_width: match self {
                    Button::Inactive | Button::Todo => 3.0,
                    _ => 0.0,
                },
                text_color: match self {
                    Button::Inactive | Button::Active => Color::BLACK,
                    Button::Todo | Button::Done => Color::BLACK,
                    _ => Color::WHITE,
                },
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}
