use iced::{Column, Element, Text};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::block::Block;
use crate::style::TEXT_XLARGE;
use crate::task::Message;

pub use standard::{MultiChoice, SingleChoice, StdAnswer, StdQuestion};

pub trait Question<T>: Debug + Clone + Serialize + Send + 'static
where
    T: Block,
{
    fn id(&self) -> String;
    fn description(&self) -> String;

    fn options(&self) -> Vec<String>;
    fn update(&mut self, value: T::Answer);

    fn response(&self) -> String;

    fn view(&mut self) -> Element<Message<T>> {
        Column::new().into()
    }

    fn summary(&self) -> Summary {
        Summary {
            id: self.id(),
            description: self.description(),
            options: self.options(),
            selection: self.response(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    id: String,
    description: String,
    options: Vec<String>,
    selection: String,
}

mod standard {
    use super::*;
    use iced::{Align, Checkbox, Radio};
    use serde::{Deserialize, Serialize};
    use std::fmt::Display;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct SingleChoice {
        id: String,
        description: String,
        options: Vec<String>,
        selected: Option<usize>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct MultiChoice {
        id: String,
        description: String,
        options: Vec<String>,
        selected: Vec<bool>,
    }

    impl SingleChoice {
        pub fn new<T: Display>(id: &str, description: &str, options: &[T]) -> Self {
            SingleChoice {
                id: String::from(id),
                description: String::from(description),
                options: options.into_iter().map(|x| format!("{}", x)).collect(),
                selected: None,
            }
        }
    }

    impl MultiChoice {
        pub fn new<T: Display>(id: &str, description: &str, options: &[T]) -> Self {
            MultiChoice {
                id: String::from(id),
                description: String::from(description),
                options: options.into_iter().map(|x| format!("{}", x)).collect(),
                selected: vec![false; options.len()],
            }
        }
    }

    impl<T> Question<T> for SingleChoice
    where
        T: Block<Answer = StdAnswer>,
    {
        fn id(&self) -> String {
            self.id.clone()
        }

        fn description(&self) -> String {
            self.description.clone()
        }

        fn options(&self) -> Vec<String> {
            self.options.to_vec()
        }

        fn update(&mut self, value: T::Answer) {
            self.selected = match value {
                StdAnswer::SingleChoice(v) => v,
                _ => panic!("Bad answer for single choice question"),
            }
        }

        fn response(&self) -> String {
            match self.selected {
                Some(i) => format!("{:?}", self.options[i]),
                None => format!("[]"),
            }
        }

        fn view(&mut self) -> Element<Message<T>> {
            let mut radios = Column::new().spacing(30).align_items(Align::Start);

            for (i, option) in self.options.to_vec().into_iter().enumerate() {
                radios = radios.push(
                    Radio::new(
                        i, //self.selected == Some(i),
                        option,
                        self.selected,
                        move |_| Message::<T>::UpdateResponse(StdAnswer::SingleChoice(Some(i))),
                    )
                    // .width(Length::Units(450))
                    .text_size(TEXT_XLARGE),
                );
            }

            Column::new()
                .spacing(60)
                .align_items(Align::Center)
                .push(Text::new(&self.description).size(TEXT_XLARGE))
                .push(radios)
                .into()
        }
    }

    impl<T> Question<T> for MultiChoice
    where
        T: Block<Answer = StdAnswer>,
    {
        fn id(&self) -> String {
            self.id.clone()
        }

        fn description(&self) -> String {
            self.description.clone()
        }

        fn options(&self) -> Vec<String> {
            self.options.to_vec()
        }

        fn update(&mut self, value: T::Answer) {
            self.selected = match value {
                StdAnswer::MultiChoice(v) => v,
                _ => panic!("Bad answer for single choice question"),
            };
        }

        fn response(&self) -> String {
            let selection: Vec<_> = self
                .options
                .iter()
                .enumerate()
                .filter_map(|(i, v)| if self.selected[i] { Some(v) } else { None })
                .collect();

            format!("{:?}", selection)
        }

        fn view(&mut self) -> Element<Message<T>> {
            let mut checkboxes = Column::new().spacing(30).align_items(Align::Start);

            for (i, option) in self.options.to_vec().into_iter().enumerate() {
                let selected = self.selected.clone();
                checkboxes = checkboxes.push(
                    Checkbox::new(selected[i], option, move |checked| {
                        let mut value = selected.clone();
                        value[i] = checked;
                        Message::<T>::UpdateResponse(StdAnswer::MultiChoice(value))
                    })
                    // .width(Length::Units(450))
                    .text_size(TEXT_XLARGE),
                );
            }

            Column::new()
                .spacing(60)
                .align_items(Align::Center)
                .push(Text::new(&self.description).size(TEXT_XLARGE))
                .push(checkboxes)
                .into()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum StdQuestion {
        SingleChoice(SingleChoice),
        MultiChoice(MultiChoice),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum StdAnswer {
        SingleChoice(Option<usize>),
        MultiChoice(Vec<bool>),
    }

    impl<T> Question<T> for StdQuestion
    where
        T: Block<Answer = StdAnswer>,
    {
        fn id(&self) -> String {
            match self {
                StdQuestion::SingleChoice(q) => <SingleChoice as Question<T>>::id(q),
                StdQuestion::MultiChoice(q) => <MultiChoice as Question<T>>::id(q),
            }
        }

        fn description(&self) -> String {
            match self {
                StdQuestion::SingleChoice(q) => <SingleChoice as Question<T>>::description(q),
                StdQuestion::MultiChoice(q) => <MultiChoice as Question<T>>::description(q),
            }
        }

        fn options(&self) -> Vec<String> {
            match self {
                StdQuestion::SingleChoice(q) => <SingleChoice as Question<T>>::options(q),
                StdQuestion::MultiChoice(q) => <MultiChoice as Question<T>>::options(q),
            }
        }

        fn update(&mut self, value: T::Answer) {
            match self {
                StdQuestion::SingleChoice(q) => {
                    <SingleChoice as Question<T>>::update(q, value);
                }
                StdQuestion::MultiChoice(q) => {
                    <MultiChoice as Question<T>>::update(q, value);
                }
            }
        }

        fn response(&self) -> String {
            match self {
                StdQuestion::SingleChoice(q) => <SingleChoice as Question<T>>::response(q),
                StdQuestion::MultiChoice(q) => <MultiChoice as Question<T>>::response(q),
            }
        }

        fn view(&mut self) -> Element<Message<T>> {
            match self {
                StdQuestion::SingleChoice(q) => <SingleChoice as Question<T>>::view(q),
                StdQuestion::MultiChoice(q) => <MultiChoice as Question<T>>::view(q),
            }
        }
    }

    impl StdQuestion {
        pub fn single_choice(id: &str, description: &str, options: &[&str]) -> Self {
            StdQuestion::SingleChoice(SingleChoice::new(id, description, options))
        }

        pub fn multi_choice(id: &str, description: &str, options: &[&str]) -> Self {
            StdQuestion::MultiChoice(MultiChoice::new(id, description, options))
        }
    }
}
