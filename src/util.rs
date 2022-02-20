pub fn timestamp() -> String {
    let time = chrono::Utc::now();
    let millis = time.timestamp_subsec_millis();
    format!("{}-{:02}-UTC", time.format("%Y-%m-%d-%H-%M-%S"), millis)
}
