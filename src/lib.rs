use std::fs::File;

/// Helper function to create file plus its parents (panics on err)
pub fn create_data_file(path: &str) -> File {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent dir")
    }
    std::fs::File::create(path).expect("failed to create output file")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
