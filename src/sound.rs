use rodio::{Decoder, OutputStream, Sample, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

use crate::comm::{Comm, Message};

pub fn play_audio(comm: Comm, src: &Path, trigger: Option<&Path>) -> Result<(), ()> {
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to open output stream");

    let sink = Sink::try_new(&stream_handle).expect("Failed to open sink stream");

    println!("Playing audio file: {:?}", src);
    let file = BufReader::new(File::open(src).unwrap());
    let source = Decoder::new(file).unwrap();

    match trigger {
        Some(path) => {
            println!("Using trigger file: {:?}", path);
            let file = BufReader::new(File::open(path).unwrap());
            let trigger = Decoder::new(file).unwrap();
            sink.append(Triggered::new(source, trigger))
        }
        None => {
            sink.append(source);
        }
    }

    while !sink.empty() {
        thread::sleep(Duration::from_millis(500));
        match comm.1.try_recv() {
            Ok(Message::Interrupt) | Err(TryRecvError::Disconnected) => {
                sink.stop();
                return Err(());
            },
            Err(TryRecvError::Empty) => (),
            _ => panic!("Unexpected message received"),
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Triggered<I>
where
    I: Source,
    I::Item: Sample,
{
    input: I,
    trigger: I,
    current_channel: u16,
}

impl<I> Triggered<I>
where
    I: Source,
    I::Item: Sample,
{
    pub fn new(input: I, trigger: I) -> Triggered<I>
    where
        I: Source,
        I::Item: Sample,
    {
        assert_eq!(
            input.channels(),
            1,
            "When using a trigger, audio signal should be mono"
        );
        assert_eq!(trigger.channels(), 1, "The trigger signal should be mono");
        assert_eq!(
            input.sample_rate(),
            trigger.sample_rate(),
            "Sampling rate of audio and trigger should be equal"
        );
        assert_eq!(
            input.total_duration(),
            trigger.total_duration(),
            "Duration of audio and trigger should be equal"
        );

        Triggered {
            input,
            trigger,
            current_channel: 0,
        }
    }

    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    /// Returns the inner source.
    #[inline]
    pub fn into_inner(self) -> I {
        self.input
    }
}

impl<I> Iterator for Triggered<I>
where
    I: Source,
    I::Item: Sample,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        // let weight = 1.0 / self.input.channels() as f32;
        if self.current_channel == 0 {
            // let mut sample = I::Item::zero_value();
            // for _ in 0..self.input.channels() {
            //     if let Some(s) = self.input.next() {
            //         sample = sample.saturating_add(s.amplify(weight));
            //     } else {
            //         return None;
            //     }
            // }

            self.current_channel = 1;
            self.input.next() // Some(sample)
        } else {
            self.current_channel = 0;
            self.trigger.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl<I> ExactSizeIterator for Triggered<I>
where
    I: Source + ExactSizeIterator,
    I::Item: Sample,
{
}

impl<I> Source for Triggered<I>
where
    I: Source,
    I::Item: Sample,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        2
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }
}
