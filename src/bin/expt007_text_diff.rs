use migration_rs::*;
use std::{env::args, path::PathBuf};
use text_diff::diff;

pub const ANSI_ESCAPE_COLOR_GREEN: &str = "\x1b[32m";
pub const ANSI_ESCAPE_COLOR_RED: &str = "\x1b[31m";
pub const ANSI_ESCAPE_COLOR_YELLOW: &str = "\x1b[33m";
pub const ANSI_ESCAPE_RESET: &str = "\x1b[0m";

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

    // let split: String = args()
    //     .nth(3)
    //     .expect("Please provide split string");

    let content1 =
        common::read_file_content(&path1).expect("Failed to read content for first path");

    let content2 =
        common::read_file_content(&path2).expect("Failed to read content for second path");

    let (dist, changeset) = diff(&content1, &content2, "");

    println!("dist: {dist}");

    for change in changeset {
        match change {
            text_diff::Difference::Same(s) => {
                print!("{ANSI_ESCAPE_COLOR_YELLOW}\"{s}\"{ANSI_ESCAPE_RESET}");
            }
            text_diff::Difference::Add(s) => {
                print!("{ANSI_ESCAPE_COLOR_GREEN}\"{s}\"{ANSI_ESCAPE_RESET}");
            }
            text_diff::Difference::Rem(s) => {
                print!("{ANSI_ESCAPE_COLOR_RED}\"{s}\"{ANSI_ESCAPE_RESET}");
            }
        }
    }
}
