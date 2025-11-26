pub fn get_data_dir() -> std::path::PathBuf {
    let dir_name = format!("{}/data", env!("CARGO_MANIFEST_DIR"));
    let dir = std::path::PathBuf::from(dir_name);
    std::fs::create_dir_all(&dir).expect("Failed to create directory");
    dir
}
