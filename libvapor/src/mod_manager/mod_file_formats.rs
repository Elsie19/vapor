use std::{fs::File, path::Path};

use compress_tools::list_archive_files;

pub fn read_files<P: AsRef<Path>>(file: P) -> Vec<String> {
    let mut paths = vec![];
    let Ok(file) = File::open(file.as_ref()) else {
        return paths;
    };

    let Ok(archive) = list_archive_files(file) else {
        return paths;
    };

    for path in archive {
        if !path.ends_with('/') {
            paths.push(path);
        }
    }

    paths
}
