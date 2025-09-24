use migration_rs::*;
use std::{env::args, path::PathBuf};

fn main() {
    drivers::init_logging_with_level(log::LevelFilter::Trace);

    let path1 = args()
        .nth(1)
        .expect("Please provide a file path")
        .parse::<PathBuf>()
        .expect("Failed to parse file path");

    let path2 = args()
        .nth(2)
        .expect("Please provide a file path")
        .parse::<PathBuf>()
        .expect("Failed to parse file path");

    let content1 =
        common::read_file_content(&path1).expect("Failed to read content for first path");

    let content2 =
        common::read_file_content(&path2).expect("Failed to read content for second path");

    drivers::display_diff(&content1, &content2, drivers::DisplayDiffFrom::default());
}
