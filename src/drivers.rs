use std::path::PathBuf;
use std::{env::args, path::Path};

use log::LevelFilter;

use crate::{cluster_note, common::ObsidianVaultPath};

pub fn init_logging_with_level(level: LevelFilter) {
    env_logger::builder()
        .filter_level(level)
        .filter_module("rustyline", LevelFilter::Warn)
        .try_init()
        .map_err(|e| e.to_string())
        .expect("Failed to initialize logger");
}

pub fn get_obsidian_vault(n: usize) -> ObsidianVaultPath {
    let folder = args()
        .nth(n)
        .expect("Please provide a vault path")
        .parse::<PathBuf>()
        .expect("Failed to parse folder as a path");

    ObsidianVaultPath::new(&folder).expect("Folder passed is not a valid obsidian vault")
}

pub fn get_arg_note_path(n: usize) -> PathBuf {
    args()
        .nth(n)
        .expect("Please provide a note path")
        .parse::<PathBuf>()
        .expect("Failed to parse note path")
}

pub fn get_opt_arg_note_path(n: usize) -> Option<PathBuf> {
    args().nth(n)?.parse::<PathBuf>().ok()
}

pub fn process_non_peripheral_markdown_files_in_vault(
    vault_folder: &ObsidianVaultPath,
    process_markdown_file: impl Fn(&Path) -> Option<()>,
) {
    let working_items = cluster_note::get_working_item_paths_in_vault(vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        match item {
            cluster_note::WorkingPath::Note(normal_note_file_path) => {
                process_markdown_file(&normal_note_file_path.path);
            }
            cluster_note::WorkingPath::ClusterFolder {
                cluster_root_folder: _cluster_root_folder,
                core_note_file: _core_note_file,
                category_folders_with_peripheral_files: _category_folders_with_peripheral_files,
            } => {
                process_markdown_file(&_core_note_file.path);
            }
        }
    }
}

pub fn process_markdown_files_in_vault(
    vault_folder: &ObsidianVaultPath,
    process_markdown_file: impl Fn(&Path) -> Option<()>,
) {
    let working_items = cluster_note::get_working_item_paths_in_vault(vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        match item {
            cluster_note::WorkingPath::Note(normal_note_file_path) => {
                process_markdown_file(&normal_note_file_path.path);
            }
            cluster_note::WorkingPath::ClusterFolder {
                cluster_root_folder: _cluster_root_folder,
                core_note_file: _core_note_file,
                category_folders_with_peripheral_files: _category_folders_with_peripheral_files,
            } => {
                process_markdown_file(&_core_note_file.path);
                _category_folders_with_peripheral_files
                    .into_iter()
                    .flat_map(|(_, peripheral_files)| peripheral_files)
                    .for_each(|peripheral_file| {
                        process_markdown_file(&peripheral_file.path);
                    });
            }
        }
    }
}
