use iced::{Element, Length};
use iced_native::Svg;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, SendError, Sender};

use crate::aux::{rel_path, rel_path_from};
use crate::config::Config;
use crate::error::Error;
use crate::question::Question;
use crate::sound::play_audio;
use crate::task::Message;

pub struct Communication(Sender<()>, Receiver<()>);

impl Communication {
    pub fn two_way() -> (Communication, Communication) {
        let (tx1, rx2) = mpsc::channel();
        let (tx2, rx1) = mpsc::channel();

        (Communication(tx1, rx1), Communication(tx2, rx2))
    }

    pub fn send(&self) -> Result<(), SendError<()>> {
        self.0.send(())
    }

    pub fn recv(&self) -> Result<(), RecvError> {
        self.1.recv()
    }
}

pub trait Block: Debug + Clone + Serialize + Send + 'static {
    type Config: Config; // + 'static;
    type Question: Question<Self>; // + 'static;
    type Answer: Debug + Clone + Serialize + Send + 'static;

    fn id(&self) -> String;

    fn title(&self) -> String {
        self.id()
    }

    fn description(&self) -> String {
        "".to_string()
    }

    fn run(id: String, config: Self::Config, comm: Communication) -> Result<(), Error>;

    fn view(&mut self) -> Element<Message<Self>> {
        Svg::from_path(rel_path("resources/image/fixation-cross-small.svg"))
            .width(Length::Units(60))
            .height(Length::Units(60))
            .into()
    }

    fn questionnaire(
        _id: &str,
        _config: &Self::Config,
        // _comm: Communication
    ) -> Vec<Self::Question> {
        vec![]
    }
}

pub trait AudioBlock: Block {
    fn audio_src(id: &str) -> String;

    fn run(id: String, use_trigger: bool, comm: Communication) -> Result<(), Error> {
        eprintln!("Starting block `{}`...", id);
        let Communication(_, rx) = comm;

        let file = rel_path_from(&rel_path("resources/sound"), &Self::audio_src(&id));
        let trigger = file.with_extension("trig.wav");

        let _rx = play_audio(
            file.as_path(),
            if use_trigger {
                Some(trigger.as_path())
            } else {
                None
            },
            rx,
        )?;

        eprintln!("Completed block `{}`.", id);

        Ok(())
    }
}
