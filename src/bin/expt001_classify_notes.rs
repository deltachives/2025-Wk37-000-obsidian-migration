/// TODO

use migration_rs::cluster_note::get_working_item_paths_in_vault;
use migration_rs::*;
use std::env::args;
use std::path::PathBuf;

use fstdout_logger::{LoggerConfig, init_logger_with_config};
use log::LevelFilter;
fn main() {
    let config = LoggerConfig::builder()
        .level(LevelFilter::Trace)
        .show_date_in_stdout(true)
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

    let working_items =
        get_working_item_paths_in_vault(&vault_folder).expect("Failed to get working items");

    for item in working_items {
        match item {
            cluster_note::WorkingPath::Note(_normal_note_file_path) => {

            },
            cluster_note::WorkingPath::ClusterFolder {
                cluster_root_folder: _cluster_root_folder,
                core_note_file: _core_note_file,
                category_folders_with_peripheral_files: _category_folders_with_peripheral_files,
            } => {

            },
        }
    }
}
