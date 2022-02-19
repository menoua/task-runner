use std::env;
use std::path::PathBuf;
use iced::{Application, Settings, window};
use rodio::cpal::Sample;

use neurotask::app::App;
use neurotask::task::Task;

fn main() {
    let args = env::args();
    let task_dir = match args.len() {
        1 => env::current_exe().unwrap().parent().unwrap().to_path_buf(),
        2 => PathBuf::from(args.skip(1).next().unwrap()),
        _ => panic!("Usage example: neurotask [task_dir]"),
    };
    let task = Task::new(task_dir);

    App::run(Settings {
        default_font: None,
        default_text_size: (20.0 * task.gui().font_scale()).round() as u16,
        exit_on_close_request: true,
        antialiasing: false,
        window: window::Settings {
            size: task.gui().window_size(),
            resizable: task.gui().resizable(),
            always_on_top: false,
            icon: None,
            ..Default::default()
        },
        flags: task,
    }).unwrap();
}
