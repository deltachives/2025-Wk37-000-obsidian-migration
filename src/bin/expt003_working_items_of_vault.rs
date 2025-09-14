use std::env::args;
use std::path::PathBuf;
use migration_rs::cluster_note::get_working_item_paths_in_vault;
use migration_rs::*;

use fstdout_logger::{init_logger_with_config, LoggerConfig};
use log::LevelFilter;
fn main() {

    let config = LoggerConfig::builder()
        .level(LevelFilter::Trace)
        .use_colors(true)
        .build();

    init_logger_with_config(Some("debug.log"), config).expect("Failed to initialize logger");

    let folder = args()
        .nth(1)
        .expect("Please provide a vault path")
        .parse::<PathBuf>()
        .expect("Failed to parse folder as a path");

    let vault_folder = common::ObsidianVaultPath::new(&folder)
        .expect("Folder passed is not a valid obsidian vault");

    let working_items = get_working_item_paths_in_vault(&vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        println!("{item:?}")
    }
}
