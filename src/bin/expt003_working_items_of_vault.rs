use migration_rs::*;

fn main() {
    let vault_folder = drivers::init_logging_and_get_obsidian_vault();

    let working_items = cluster_note::get_working_item_paths_in_vault(&vault_folder)
        .expect("Failed to get working items");

    for item in working_items {
        println!("{item:?}")
    }
}
