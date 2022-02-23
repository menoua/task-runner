use std::env;
use std::path::PathBuf;
use iced::{Application, Settings, window};

use neurotask::app::App;
use neurotask::task::Task;

fn main() -> Result<(), String> {
    let args = env::args();
    let task_dir = match args.len() {
        1 => env::current_exe().unwrap().parent().unwrap().to_path_buf(),
        2 => PathBuf::from(args.skip(1).next().unwrap()),
        _ => panic!("Usage example: neurotask [task_dir]"),
    };
    let task = Task::new(task_dir)?;
    let global = task.global();
    global.verify();

    App::run(Settings {
        default_font: None,
        default_text_size: global.text_size("NORMAL"),
        exit_on_close_request: true,
        antialiasing: false,
        window: window::Settings {
            size: global.window_size(),
            min_size: global.min_window_size(),
            resizable: global.resizable(),
            always_on_top: false,
            icon: None,
            ..Default::default()
        },
        flags: task,
    }).or_else(|e| match e {
        iced::Error::GraphicsAdapterNotFound => Err(
            "A suitable graphics adapter was not found. On linux, this could mean that you \
            are missing the Vulkan graphics library. On Ubuntu, you can install the Vulkan \
            library using: `sudo apt-get install libvulkan1`.\n".to_string()
        ),
        iced::Error::ExecutorCreationFailed(_) => Err(
            "ExecutorCreationFailed".to_string()
        ),
        iced::Error::WindowCreationFailed(_) => Err(
            "WindowCreationFailed".to_string()
        ),
    })
}
