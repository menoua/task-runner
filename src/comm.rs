use std::any::TypeId;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use iced::keyboard::KeyCode;
use iced_native::subscription::Recipe;
use iced_futures::futures;

use crate::action::ID;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i32),
    Float(f32),
    Char(char),
    String(String),
}

#[derive(Debug, Clone)]
pub enum LogMode {
    Event,
    Behavior
}

pub type Code = u16;

#[derive(Debug, Clone)]
pub enum Message {
    Code(ID, ID, Code),
    Value(ID, ID, Code, Value),
    UIEvent(Code, Value),
    KeyPress(KeyCode),
    Log(LogMode, String),
    SetComms(Sender),
    Interrupt,
    Query(ID, String),
    QueryResponse(ID, String),
    ActionComplete(ID),
    BlockComplete,
    Wrap,
    Null,
}

pub type Sender = mpsc::Sender<Message>;
pub type Receiver = mpsc::Receiver<Message>;
pub type Comm = (Sender, Receiver);

pub struct CommLink {
    writer: Sender,
    inbox: Receiver,
    is_ready: bool,
}

impl CommLink {
    pub fn new() -> Self {
        let (writer, inbox) = mpsc::channel();
        CommLink { writer, inbox, is_ready: false }
    }

    pub fn new_writer(&self) -> Sender {
        self.writer.clone()
    }
}

// Make sure iced can use our download stream
impl<H, I> Recipe<H, I> for CommLink
    where
        // T: 'static + Hash + Copy + Send,
        H: Hasher,
{
    type Output = Message;

    fn hash(&self, state: &mut H) {
        struct Marker;
        TypeId::of::<Marker>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            self,
            |mut comm_link| async {
                if !comm_link.is_ready {
                    comm_link.is_ready = true;
                    Some((Message::SetComms(comm_link.new_writer()), comm_link))
                } else {
                    match comm_link.inbox.try_recv() {
                        Ok(message) => {
                            Some((message, comm_link))
                        },
                        Err(TryRecvError::Empty) => {
                            std::thread::sleep(Duration::from_millis(1));
                            Some((Message::Null, comm_link))
                        },
                        Err(TryRecvError::Disconnected) => {
                            panic!("Dispatcher has died!!")
                        },
                    }
                }
            },
        ))
    }
}
