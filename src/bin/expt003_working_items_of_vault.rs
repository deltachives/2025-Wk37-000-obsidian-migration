use migration_rs::*;

fn main() {
    drivers::init_logging_with_level(log::LevelFilter::Trace);

    let vault_folder = drivers::get_obsidian_vault(1);

    let working_items = cluster_note::get_working_item_paths_in_vault(&vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        println!("{item:?}")
    }
}
