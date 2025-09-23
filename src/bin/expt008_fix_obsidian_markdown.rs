use migration_rs::*;
use std::{env::args, path::PathBuf};
use tap::prelude::*;

fn main() {
    drivers::init_logging_with_level(log::LevelFilter::Trace);

    let path = args()
        .nth(1)
        .expect("Please provide a file path")
        .parse::<PathBuf>()
        .expect("Failed to parse file path");

    let content = common::read_file_content(&path).expect("Failed to read content for first path");

    let events = common::parse_markdown_file(&content);

    let new_content = common::render_events_to_common_markdown(&events)
        .expect("Failed to render back to common markdown")
        .pipe(|new_content| {
            common::adhoc_fix_rendered_markdown_output_for_obsidian(&content, &new_content)
        });

    println!("{new_content}");
}
