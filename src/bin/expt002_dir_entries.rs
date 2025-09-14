use std::env::args;
use std::path::PathBuf;

fn main() {
    let folder = args()
        .nth(1)
        .expect("Please provide a folder path")
        .parse::<PathBuf>()
        .expect("Failed to parse folder as a path");

    let dir = folder.read_dir().unwrap();

    let dir_entries = dir.collect::<Result<Vec<_>, _>>().unwrap();

    let folder_entries = dir_entries
        .iter()
        .filter(|entry| entry.metadata().unwrap().is_dir())
        .collect::<Vec<_>>();

    let file_entries = dir_entries
        .iter()
        .filter(|entry| entry.metadata().unwrap().is_file())
        .collect::<Vec<_>>();

    println!("{folder_entries:?}\n");
    println!("{file_entries:?}");
}
