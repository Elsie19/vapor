use std::{fs::File, path::Path};

use zip::ZipArchive;

pub fn read_files<P: AsRef<Path>>(file: P) -> Vec<String> {
    let mut paths = vec![];
    let Ok(file) = File::open(file.as_ref()) else {
        return paths;
    };

    let Some(mut archive) = ZipArchive::new(file).ok() else {
        return paths;
    };

    for i in 0..archive.len() {
        let file = archive.by_index(i).expect("Oops");
        if !file.is_dir() {
            paths.push(file.name().to_string());
        }
    }

    paths
}
