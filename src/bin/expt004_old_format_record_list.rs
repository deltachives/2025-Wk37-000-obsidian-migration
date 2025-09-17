use migration_rs::*;
use std::path::PathBuf;

fn main() {
    let vault_folder = drivers::init_logging_and_get_obsidian_vault();
    let opt_note_path = drivers::get_opt_arg_note_path();

    let process_markdown_file = |path: &PathBuf, only_summarize: bool| -> Option<()> {
        let content = common::read_file_content(path).expect("Could not read content");

        let events = common::parse_markdown_file(&content);

        let old_format_records = cluster_note::get_note_old_format_entries(&events).ok()?;

        let records_with_extracts = old_format_records
            .iter()
            .map(|entry| {
                (
                    entry,
                    common::extract_linkable_obsidian_md_items(&entry.events),
                    common::extract_obsidian_md_links(&entry.events)
                        .expect("Failed to extract obsidian links"),
                )
            })
            .collect::<Vec<_>>();

        if !records_with_extracts.is_empty() {
            log::trace!(
                "\n\n\n---\npath {path:?} has {} records",
                records_with_extracts.len()
            );
        }

        for (record, linkables, links) in records_with_extracts {
            let spawn_metadata =
                cluster_note::extract_spawn_metadata_from_old_format(&linkables, &links);

            log::trace!(
                "record {:?} of {:?} has {} events.",
                record.entry_name,
                record.entry_type,
                record.events.len()
            );
            log::trace!(
                "\tIt has {} linkables, {} links, and {} spawn items.",
                linkables.len(),
                links.len(),
                spawn_metadata.len()
            );

            if !only_summarize {
                log::trace!("record: {record:?}");
                log::trace!("linkables: {linkables:?}");
                log::trace!("links: {links:?}");
                log::trace!("spawn_metadata: {spawn_metadata:?}\n\n");
            }
        }

        Some(())
    };

    match opt_note_path {
        Some(note_path) => {
            process_markdown_file(&note_path, false);
        }
        None => {
            drivers::process_markdown_files_in_vault(&vault_folder, |path| {
                process_markdown_file(path, true)
            });
        }
    }
}
