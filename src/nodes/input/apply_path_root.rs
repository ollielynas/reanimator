use std::{path::PathBuf, str::FromStr};

use crate::storage::Storage;

pub fn set_root(path: &mut PathBuf, storage: &Storage) {
    if path.starts_with(&storage.project_root) {
        let s = path
            .display()
            .to_string()
            .replace(&storage.project_root.display().to_string(), "PROJECT_ROOT");
        *path = PathBuf::from_str(&s).unwrap_or_else(|_| path.clone());
    }
}

pub fn get_with_root(path: &PathBuf, storage: &Storage) -> PathBuf {
    if path.starts_with(PathBuf::from_str("PROJECT_ROOT").unwrap_or_default()) {
        let s = path
            .display()
            .to_string()
            .replace("PROJECT_ROOT", &storage.project_root.display().to_string());
        return PathBuf::from_str(&s).unwrap_or_else(|_| path.clone());
    } else {
        return path.clone();
    }
}
