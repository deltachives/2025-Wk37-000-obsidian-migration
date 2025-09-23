/// TODO
use migration_rs::*;

fn main() {
    drivers::init_logging_with_level(log::LevelFilter::Trace);

    let vault_folder = drivers::get_obsidian_vault(1);

    let working_items = cluster_note::get_working_item_paths_in_vault(&vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        match item {
            cluster_note::WorkingPath::Note(_normal_note_file_path) => {}
            cluster_note::WorkingPath::ClusterFolder {
                cluster_root_folder: _cluster_root_folder,
                core_note_file: _core_note_file,
                category_folders_with_peripheral_files: _category_folders_with_peripheral_files,
            } => {}
        }
    }
}
