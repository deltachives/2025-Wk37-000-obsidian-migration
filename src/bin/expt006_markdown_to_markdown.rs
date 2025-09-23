//! Takes a common markdown file, parses it, and outputs its content back from the parsed events

use migration_rs::*;
use pulldown_cmark::{Parser, TextMergeStream};

fn main() {
    drivers::init_logging_with_level(log::LevelFilter::Trace);

    let path = drivers::get_arg_note_path(1);

    let content = common::read_file_content(&path).expect("Could not read file to string");

    let parser = Parser::new(&content);

    let iter = TextMergeStream::new(parser);

    let events = iter.collect::<Vec<_>>();

    let out = common::render_events_to_common_markdown(&events)
        .expect("Could not render back to markdown");

    println!("{}", out);
}
