use std::path::Path;

pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    std::fs::create_dir_all(path.as_ref()).map_err(|e| e.to_string())
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    std::fs::read_to_string(path.as_ref()).map_err(|e| e.to_string())
}

pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), String> {
    std::fs::write(path.as_ref(), contents).map_err(|e| e.to_string())
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
    std::fs::remove_file(path.as_ref()).map_err(|e| e.to_string())
}
