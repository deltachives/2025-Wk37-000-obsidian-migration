//! This example takes a markdown input and logs pulldown cmark events in it

use pulldown_cmark::{Event, Parser, TextMergeStream};
use std::{env::args, fs::File, io::Read, path::PathBuf};

fn main() {
    let path = args()
        .nth(1)
        .expect("Must provide a markdown path")
        .parse::<PathBuf>()
        .expect("Invalid path");

    let content = {
        let mut file = File::open(&path).expect("Could not open file");
        let mut out = String::new();

        file.read_to_string(&mut out)
            .expect("Could not read file to string");

        out
    };

    let parser = Parser::new(&content);

    let iter = TextMergeStream::new(parser);

    for event in iter {
        println!("Event {event:?}");

        match event {
            Event::Start(_tag) => {}
            Event::End(_tag_end) => {}
            Event::Text(_cow_str) => {}
            Event::Code(_cow_str) => {}
            Event::InlineMath(_cow_str) => {}
            Event::DisplayMath(_cow_str) => {}
            Event::Html(_cow_str) => {}
            Event::InlineHtml(_cow_str) => {}
            Event::FootnoteReference(_cow_str) => {}
            Event::SoftBreak => {}
            Event::HardBreak => {}
            Event::Rule => {}
            Event::TaskListMarker(_) => {}
        }
    }
}
