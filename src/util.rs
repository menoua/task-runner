use std::fs::File;
use std::path::{Path, PathBuf};
use serde::Serialize;

pub fn timestamp() -> String {
    let time = chrono::Utc::now();
    let millis = time.timestamp_subsec_millis();
    format!("{}-{:02}-UTC", time.format("%Y-%m-%d-%H-%M-%S"), millis)
}

pub fn resource(task_dir: &Path, file: &str) -> Result<PathBuf, String> {
    let mut path = task_dir.join("resources").to_path_buf();
    for part in file.split('/') {
        path = path.join(part);
    }
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("Resource file not found: {}", path.to_str().unwrap()))
    }
}

pub fn template(task_dir: &Path, file: &str) -> Result<PathBuf, String> {
    let mut path = task_dir.join("templates").to_path_buf();
    for part in file.split('/') {
        path = path.join(part);
    }
    if path.extension().is_some() && path.exists() {
        Ok(path)
    } else if path.with_extension("yml").exists() {
        Ok(path.with_extension("yml"))
    } else {
        Err(format!("Template file not found: {}", path.to_str().unwrap()))
    }
}

pub fn output(log_dir: &str, id: &str) -> String {// Result<PathBuf, String> {
    // let mut path = task_dir.join("output").to_path_buf();
    // for part in file.split('/') {
    //     path = path.join(part);
    // }
    Path::new(log_dir)
        .join(format!("action-{}-{}", id, timestamp()))
        .to_str().unwrap().to_string()
}

pub fn async_write_to_file<T>(filename: String, data: T, err: &'static str)
where
    T: Send + Serialize + 'static
{
    std::thread::spawn(move || {
        let file = File::create(filename).unwrap();
        serde_yaml::to_writer(file, &data).expect(err);
    });
}
