pub fn timestamp() -> String {
    let time = chrono::Utc::now();
    let millis = (time.timestamp_subsec_millis() as f32 / 10.0).round() as u32;
    format!("{}-{:02}-UTC", time.format("%Y-%m-%d-%H-%M-%S"), millis)
}
