use log::*;
use migration_rs::{
    cluster_note::{ClusterFolderPath, CoreNoteFilePath},
    common::ObsidianVaultPath,
    *,
};
use std::path::{Path, PathBuf};
use tap::prelude::*;

use clap::{Arg, ArgAction, ArgMatches, Command, arg, command, value_parser};

fn parse_args() -> ArgMatches {
    command!()
        .arg(
            Arg::new("verbose")
                .help("-v: debug, -vv: trace")
                .short('v')
                .action(ArgAction::Count),
        )
        .subcommand(
            Command::new("writeback")
                .about("Parses and rewrites markdown files to the vault which includes some minor changes like line trims")
                .arg(
                    arg!([vault_path] "Path to the vault")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("extract_old_format_records")
                .about("Extracts old format records into peripheral notes, replacing spawned events")
                .arg(
                    arg!([vault_path] "Path to the vault")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand_required(true)
        .get_matches()
}

fn skip_processing_managed_path(path: &Path) -> bool {
    let filename = path.file_name().unwrap().to_string_lossy();
    let pathname = format!("{path:?}");

    filename.contains("excalidraw")
        || filename.contains("Kanban")
        || filename.contains("Summarize")
        || filename.contains("Summary")
        || filename.contains("Timeline")
        || pathname.contains("templater")
}

fn app_writeback(vault_path: &ObsidianVaultPath) {
    let process_markdown_file = |path: &Path| -> Option<()> {
        // Some markdown files managed by extensions and should be skipped
        if skip_processing_managed_path(path) {
            return Some(());
        }

        let content = common::read_file_content(path).expect("Could not read content");

        let events = common::parse_markdown_file(&content);

        let new_content = common::render_events_to_common_markdown(&events)
            .expect("Failed to render back to common markdown")
            .pipe(|new_content| {
                common::adhoc_fix_rendered_markdown_output_for_obsidian(&content, &new_content)
            });

        common::write_file_content(&new_content, path).expect("Failed to write file content");

        Some(())
    };

    drivers::process_markdown_files_in_vault(vault_path, process_markdown_file);
}

fn app_extract_old_format_records(vault_path: &ObsidianVaultPath) {
    info!("value_path: {vault_path:?}");

    let process_markdown_file = |path: &Path| -> Option<()> {
        // Some markdown files managed by extensions and should be skipped
        if skip_processing_managed_path(path) {
            return Some(());
        }

        // TODO: Temporary for debugging
        // if let Some(filename) = path.file_name()
        //     && let Some(s) = filename.to_str()
        //     && !s.contains("Attention is all you need")
        // {
        //     return Some(());
        // }

        info!("processing {path:?}");

        let content = common::read_file_content(path).expect("Could not read content");

        let events = common::parse_markdown_file(&content);

        let (remaining_content, old_format_records) =
            cluster_note::get_note_old_format_entries_from_content(&content)
                .expect("Failed to process old format entries from content");

        if old_format_records.is_empty() {
            return None;
        }

        let linkables = common::extract_linkable_obsidian_md_items(&events);

        let links = common::extract_obsidian_md_links(&events).expect("Could not process links");

        let _spawn_metadata =
            cluster_note::extract_spawn_metadata_from_old_format(&linkables, &links);

        // Are we in a cluster note already? if not, create one for this note by its name and replace its content
        // with content that includes no old entries

        let core_note_path = {
            let opt_core_note_path = CoreNoteFilePath::new(path);

            match opt_core_note_path {
                Some(core_note_path) => core_note_path,
                None => cluster_note_io::turn_note_into_cluster_note(path)
                    .expect("Failed to turn note into cluster note"),
            }
        };

        let cluster_folder_path = ClusterFolderPath::new(
            core_note_path
                .path
                .parent()
                .expect("core note must have a parent"),
        )
        .expect("Cluster folder must be known if core note is known");

        common::write_file_content(&remaining_content, &core_note_path.path)
            .expect("Failed to write file content");

        // Write the extracted entries

        for rec in old_format_records {
            cluster_note_io::create_new_peripheral_note_from_old_format_entry(
                &cluster_folder_path,
                &rec,
            )
            .expect("Failed to create new peripheral note");
        }

        // Fixes in each file:

        // Remove all "From ..." spawned events before writing the new entries to file

        // Write the entries to file, and parent, then if available, spawned by/spawned in paths

        Some(())
    };

    drivers::process_non_peripheral_markdown_files_in_vault(vault_path, process_markdown_file);
}

fn main() {
    let matches = parse_args();

    let verbose = matches.get_count("verbose").pipe(|n| match n {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    });

    drivers::init_logging_with_level_or_fail(verbose);

    match matches.subcommand() {
        Some(("writeback", sub_matches)) => {
            let vault_path = sub_matches
                .get_one::<PathBuf>("vault_path")
                .unwrap()
                .pipe(|path| ObsidianVaultPath::new(path))
                .expect("vault path should be valid");

            app_writeback(&vault_path);
        }

        Some(("extract_old_format_records", sub_matches)) => {
            let vault_path = sub_matches
                .get_one::<PathBuf>("vault_path")
                .unwrap()
                .pipe(|path| ObsidianVaultPath::new(path))
                .expect("vault path should be valid");

            app_extract_old_format_records(&vault_path);
        }

        _ => unreachable!(),
    }
}
