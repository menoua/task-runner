use iced::{Application, Clipboard, Column, Command, Container, Element, Length, Row, Space, Subscription, Color};
use iced_native::subscription;
use std::time::{Duration, Instant};

use crate::task::Task;
use crate::comm::{Message, CommLink};

pub struct App
{
    task: Task,
    last_esc: Instant,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = Task;

    fn new(task: Task) -> (App, Command<Self::Message>) {
        println!(">> {}", task.title());

        let app = App {
            task,
            last_esc: Instant::now(),
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        self.task.title()
    }

    fn update(&mut self, message: Self::Message, _: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::Null => {
                Command::none()
            }
            Message::Interrupt => {
                let now = Instant::now();
                if now.duration_since(self.last_esc) < Duration::from_millis(250) {
                    self.task.update(message)
                } else {
                    self.last_esc = now;
                    Command::none()
                }
            }
            message => {
                self.task.update(message)
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::Event::KeyPressed;
        use iced::keyboard::KeyCode::Escape;
        use iced_native::Event::Keyboard;

        let mut subscriptions = vec![];
        if !self.task.has_dispatcher() {
            subscriptions.push(Subscription::from_recipe(CommLink::new()));
        }
        subscriptions.push(
            subscription::events_with(|event, _| match event {
                Keyboard(KeyPressed { key_code: Escape, .. }) => {
                    Some(Message::Interrupt)
                },
                Keyboard(KeyPressed { key_code, .. }) => {
                    Some(Message::KeyPress(key_code))
                },
                _ => None,
            })
        );
        Subscription::batch(subscriptions)
    }

    fn view(&mut self) -> Element<Message> {
        let content = Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Space::with_width(Length::FillPortion(1)))
            .push(self.task.view().width(Length::FillPortion(8)))
            .push(Space::with_width(Length::FillPortion(1)));

        let content = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Space::with_height(Length::FillPortion(1)))
            .push(content.height(Length::FillPortion(8)))
            .push(Space::with_height(Length::FillPortion(1)));

        let content: Element<_> = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();

        content//.explain(Color::BLACK)
    }
}
