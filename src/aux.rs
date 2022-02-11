use std::env::current_exe;
use std::path::{Path, PathBuf};

pub fn root_dir() -> PathBuf {
    current_exe().unwrap().parent().unwrap().to_path_buf()
}

pub fn rel_path(path: &str) -> PathBuf {
    let mut path_buf = root_dir();
    for part in path.split("/") {
        path_buf.push(part);
    }
    path_buf
}

pub fn rel_path_from(root: &Path, path: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(root);
    for part in path.split("/") {
        path_buf.push(part);
    }
    path_buf
}
