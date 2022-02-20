use std::fs::File;
use serde::Serialize;

pub fn timestamp() -> String {
    let time = chrono::Utc::now();
    let millis = time.timestamp_subsec_millis();
    format!("{}-{:02}-UTC", time.format("%Y-%m-%d-%H-%M-%S"), millis)
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
