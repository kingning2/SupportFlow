use std::path::PathBuf;

pub fn crate_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

pub fn project_dirs() -> Result<directories::ProjectDirs, String> {
    directories::ProjectDirs::from("com", "polymerization", "gybte")
        .ok_or_else(|| "could not resolve project dirs".to_string())
}

pub fn logs_root() -> Result<PathBuf, String> {
    Ok(project_dirs()?.data_local_dir().join("logs"))
}
