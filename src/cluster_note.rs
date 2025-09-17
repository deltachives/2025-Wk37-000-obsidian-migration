use std::{
    cmp::min,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::common::{
    self as comm, BlockIdentifier, CategorizedDirEntry, GetEventText, GetEventTextInternalError,
    ObsidianLink, ObsidianLinkItem, ObsidianLinkableItem,
};

use itertools::Itertools;
use pulldown_cmark::{Event, HeadingLevel, Tag};
use tap::prelude::*;
use thiserror::Error;

pub const NUM_EXPECTED_FOLDERS: usize = 7;
pub const _NUM_OLD_FORMAT_HEADINGS: usize = 6;

pub const CONTEXT_TYPE_FOLDERS: [&str; NUM_EXPECTED_FOLDERS] = [
    "entries",
    "howtos",
    "ideas",
    "inferences",
    "investigations",
    "issues",
    "tasks",
];

pub const CONTEXT_TYPE_BLOCK_IDENTIFIER_CODE: [&str; NUM_EXPECTED_FOLDERS] =
    ["entry", "howto", "idea", "infer", "invst", "issue", "task"];

pub const CONTEXT_TYPE_HEADINGS_SINGULAR: [&str; NUM_EXPECTED_FOLDERS] = [
    "Entry",
    "HowTo",
    "Idea",
    "Inference",
    "Investigation",
    "Issue",
    "Task",
];

pub const CONTEXT_TYPE_HEADINGS: [&str; NUM_EXPECTED_FOLDERS] = [
    "Entries",
    "HowTos",
    "Ideas",
    "Inferences",
    "Investigations",
    "Issues",
    "Tasks",
];

pub const OLD_FORMAT_HEADINGS: [&str; _NUM_OLD_FORMAT_HEADINGS] = [
    "Tasks",
    "Issues",
    "HowTos",
    "Investigations",
    "Ideas",
    "Side Notes",
];

pub fn context_type_is_doer(context_type_id: usize) -> bool {
    if context_type_id > NUM_EXPECTED_FOLDERS {
        return false;
    }

    let doer_context_type_headings = [
        // "Entries",
        "HowTos",
        // "Ideas",
        // "Inferences",
        "Investigations",
        "Issues",
        "Tasks",
    ];

    doer_context_type_headings
        .into_iter()
        .any(|heading| heading == CONTEXT_TYPE_HEADINGS[context_type_id])
}

pub fn file_exists_in_folder_of_same_name(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    let inner_fn = move || {
        let parent_folder = path.parent()?;

        let file_stem = path.file_stem()?;

        Some(path.is_file() && path.exists() && parent_folder.ends_with(file_stem))
    };

    inner_fn().unwrap_or_default()
}

pub fn is_cluster_root_folder(folder: &Path) -> Option<bool> {
    if !comm::folder_has_file_of_same_name(folder)? {
        return Some(false);
    }

    if comm::get_folder_child_file_count_non_recursive(folder)? != 1 {
        return Some(false);
    }

    Some(true)
}

pub fn is_cluster_category_folder(folder: &Path) -> Option<bool> {
    let folder_name = folder.file_name()?.to_str()?;

    if !CONTEXT_TYPE_FOLDERS.contains(&folder_name) {
        return Some(false);
    }

    if !is_cluster_root_folder(folder.parent()?)? {
        return Some(false);
    }

    Some(true)
}

pub fn is_markdown_file_path(path: &Path) -> bool {
    if !path.is_file() || !path.exists() {
        return false;
    }

    let opt_ext = path.extension();

    if let Some(ext) = opt_ext {
        if ext != "md" {
            return false;
        }
    } else {
        return false;
    }

    true
}

pub fn is_cluster_core_file_path(path: &Path) -> Option<bool> {
    if !is_markdown_file_path(path) {
        return Some(false);
    }

    let parent = path.parent()?.to_path_buf();

    if !is_cluster_root_folder(&parent)? {
        return Some(false);
    }

    Some(true)
}

pub fn is_cluster_peripheral_file_path(path: &Path) -> Option<bool> {
    if !is_markdown_file_path(path) {
        return Some(false);
    }

    let parent = path.parent()?.to_path_buf();

    if !is_cluster_category_folder(&parent)? {
        return Some(false);
    }

    Some(true)
}

pub fn is_normal_markdown_file_path(path: &Path) -> Option<bool> {
    if !is_markdown_file_path(path) {
        return Some(false);
    }

    if is_cluster_core_file_path(path)? || is_cluster_peripheral_file_path(path)? {
        return Some(false);
    }

    Some(true)
}

#[derive(Debug)]
pub struct ClusterRootFolderPath {
    pub path: PathBuf,
}

impl ClusterRootFolderPath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_cluster_root_folder(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct ClusterCategoryFolderPath {
    pub path: PathBuf,
}

impl ClusterCategoryFolderPath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_cluster_category_folder(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct CoreNoteFilePath {
    pub path: PathBuf,
}

impl CoreNoteFilePath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_cluster_core_file_path(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct PeripheralNoteFilePath {
    pub path: PathBuf,
}

impl PeripheralNoteFilePath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_cluster_peripheral_file_path(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct NormalNoteFilePath {
    pub path: PathBuf,
}

impl NormalNoteFilePath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_normal_markdown_file_path(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

pub fn get_core_note_file_from_cluster_root_folder(
    cluster_root_folder: &ClusterRootFolderPath,
) -> Option<CoreNoteFilePath> {
    let dir_entries = comm::get_and_categorize_dir_entries(&cluster_root_folder.path).ok()?;

    let core_note_path = dir_entries
        .into_iter()
        .find(|entry| matches!(entry, CategorizedDirEntry::File(_)))?
        .pipe(|path| match path {
            CategorizedDirEntry::File(dir_entry) => Some(CoreNoteFilePath::new(&dir_entry.path())?),
            _ => None,
        })?;

    Some(core_note_path)
}

pub fn get_category_folders_with_peripheral_files_from_cluster_root_folder(
    cluster_root_folder: &ClusterRootFolderPath,
) -> Option<Vec<(ClusterCategoryFolderPath, Vec<PeripheralNoteFilePath>)>> {
    let cluster_entries = comm::get_and_categorize_dir_entries(&cluster_root_folder.path).ok()?;

    let category_folders_and_periphal_files = {
        let mut mut_category_folders_and_periphal_files = vec![];

        for cluster_entry in cluster_entries {
            match cluster_entry {
                CategorizedDirEntry::Dir(category_dir_entry) => {
                    let category_folder_path =
                        ClusterCategoryFolderPath::new(&category_dir_entry.path())?;

                    let peripheral_note_files = {
                        let mut mut_peripheral_note_files = vec![];

                        // A category folder just has flat files in it.
                        let category_enries =
                            comm::get_and_categorize_dir_entries(&category_dir_entry.path())
                                .ok()?;

                        for category_entry in category_enries {
                            match category_entry {
                                CategorizedDirEntry::File(dir_entry) => {
                                    let peripheral_note_file =
                                        PeripheralNoteFilePath::new(&dir_entry.path())?;

                                    mut_peripheral_note_files.push(peripheral_note_file);
                                }

                                // There shouldn't be anything else
                                _ => return None,
                            }
                        }

                        mut_peripheral_note_files
                    };

                    mut_category_folders_and_periphal_files
                        .push((category_folder_path, peripheral_note_files));
                }
                _ => continue,
            }
        }

        mut_category_folders_and_periphal_files
    };

    Some(category_folders_and_periphal_files)
}

/// Paths of consideration for updates in the vault
#[derive(Debug)]
pub enum WorkingPath {
    Note(NormalNoteFilePath),
    ClusterFolder {
        cluster_root_folder: ClusterRootFolderPath,
        core_note_file: CoreNoteFilePath,
        category_folders_with_peripheral_files:
            Vec<(ClusterCategoryFolderPath, Vec<PeripheralNoteFilePath>)>,
    },
}

pub fn get_working_item_paths_recursive(folder: &Path) -> Option<Vec<WorkingPath>> {
    let dir_entries = comm::get_and_categorize_dir_entries(folder).ok()?;

    let items = {
        let mut mut_items = vec![];

        for dir_entry in dir_entries {
            match dir_entry {
                comm::CategorizedDirEntry::Dir(dir_entry) => {
                    let opt_cluster_root_folder = ClusterRootFolderPath::new(&dir_entry.path());

                    match opt_cluster_root_folder {
                        Some(cluster_root_folder) => {
                            let core_note_file =
                                get_core_note_file_from_cluster_root_folder(&cluster_root_folder)?;
                            let category_folders_with_peripheral_files = get_category_folders_with_peripheral_files_from_cluster_root_folder(&cluster_root_folder)?;

                            mut_items.push(WorkingPath::ClusterFolder {
                                cluster_root_folder,
                                core_note_file,
                                category_folders_with_peripheral_files,
                            });
                        }
                        None => {
                            // Keep looking!
                            mut_items.extend(get_working_item_paths_recursive(&dir_entry.path())?);
                        }
                    }
                }
                comm::CategorizedDirEntry::File(dir_entry) => {
                    // Found outside a note cluster, so it should be a normal markdown file,
                    // or just ignore it if it's an unrelated file.
                    if let Some(normal_note_file) = NormalNoteFilePath::new(&dir_entry.path()) {
                        mut_items.push(WorkingPath::Note(normal_note_file));
                    }
                }
                comm::CategorizedDirEntry::Symlink(_) => continue,
            }
        }

        mut_items
    };

    Some(items)
}

pub fn get_working_item_paths_in_vault(
    vault_folder: &comm::ObsidianVaultPath,
) -> Option<Vec<WorkingPath>> {
    get_working_item_paths_recursive(&vault_folder.path)
}

pub fn note_link_to_path(vault: &[WorkingPath], note_link: &str) -> Option<PathBuf> {
    vault
        .iter()
        .flat_map(|item| match item {
            WorkingPath::Note(normal_note_file_path) => {
                if normal_note_file_path.path.ends_with(note_link) {
                    Some(normal_note_file_path.path.clone())
                } else {
                    None
                }
            }
            WorkingPath::ClusterFolder {
                core_note_file,
                category_folders_with_peripheral_files,
                ..
            } => {
                if core_note_file.path.ends_with(note_link) {
                    Some(core_note_file.path.clone())
                } else {
                    category_folders_with_peripheral_files
                        .iter()
                        .flat_map(|(_, files)| files)
                        .flat_map(|file| {
                            if file.path.ends_with(note_link) {
                                Some(file.path.clone())
                            } else {
                                None
                            }
                        })
                        .next()
                }
            }
        })
        .next()
}

pub fn get_cluster_core_file_from_peripheral(
    vault: &[WorkingPath],
    peripheral_file: &PeripheralNoteFilePath,
) -> Option<CoreNoteFilePath> {
    let parent_note_link =
        comm::get_file_frontmatter_note_property(&peripheral_file.path, "parent")?;

    let path = note_link_to_path(vault, &parent_note_link)?;

    CoreNoteFilePath::new(&path)
}

#[derive(Debug, Clone)]
pub enum OldFormatEntryType {
    Task,
    Issue,
    HowTo,
    Investigation,
    Idea,
    SideNote,
}

#[derive(Error, Debug)]
pub enum OldFormatEntryTypeFromStrError {
    #[error("Invalid type provided: {0:?}")]
    InvalidType(String),
}

impl FromStr for OldFormatEntryType {
    type Err = OldFormatEntryTypeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "Tasks" {
            Ok(Self::Task)
        } else if s == "Issues" {
            Ok(Self::Issue)
        } else if s == "HowTos" {
            Ok(Self::HowTo)
        } else if s == "Investigations" {
            Ok(Self::Investigation)
        } else if s == "Ideas" {
            Ok(Self::Idea)
        } else if s == "Side Notes" {
            Ok(Self::SideNote)
        } else {
            Err(OldFormatEntryTypeFromStrError::InvalidType(s.to_string()))
        }
    }
}

pub fn is_autonumbered_section_segment(s: &str) -> bool {
    // all are numbers
    s.split(".")
        .map(|token| token.parse::<usize>().ok())
        .all(|token| token.is_some())
}

/// We use a section that starts headings with N.M.P.{...}. If it's found, remove it.
pub fn strip_autonumbered_sections(s: &str) -> String {
    let tokens = s.split(" ").collect::<Vec<_>>();

    if tokens.len() < 2 {
        return s.to_owned();
    }

    let potential_segt = tokens[0];

    if !is_autonumbered_section_segment(potential_segt) {
        return s.to_owned();
    }

    s.replace(potential_segt, "")
}

#[derive(Debug, Clone)]
pub struct OldFormatEntry<'a> {
    pub entry_type: OldFormatEntryType,
    pub entry_name: String,
    pub events: Vec<Event<'a>>,
}

#[derive(Error, Debug)]
pub enum GetNoteOldFormatEntriesError {
    #[error("Invalid entry type read: {0:?}")]
    InvalidEntryType(#[from] OldFormatEntryTypeFromStrError),

    #[error("Tried to parse old entry records but could not infer placement")]
    EventTypeAndNameNotConfigured,
}

pub fn get_note_old_format_entries<'a>(
    events: &'a [Event<'a>],
) -> Result<Vec<OldFormatEntry<'a>>, GetNoteOldFormatEntriesError> {
    // Let's first turn this into groups of H1/H2/SubH2
    #[derive(Debug)]
    enum Grouped<'a> {
        H1(String),
        H2(String),
        Content(&'a [Event<'a>]),
    }

    let grouped_events = {
        let mut mut_grouped_events = vec![];
        let mut mut_cur: usize = 0;

        while mut_cur < events.len() {
            match comm::process_heading_event_of_level(&HeadingLevel::H1, &events[mut_cur..]) {
                Ok(heading1) => {
                    mut_grouped_events.push(Grouped::H1(heading1));
                    mut_cur += 3;
                }
                Err(_) => {
                    match comm::process_heading_event_of_level(
                        &HeadingLevel::H2,
                        &events[mut_cur..],
                    ) {
                        Ok(heading2) => {
                            mut_grouped_events.push(Grouped::H2(heading2));
                            mut_cur += 3;
                        }
                        Err(_) => {
                            if !mut_grouped_events.is_empty() {
                                // Process all sub H2 under the same group
                                let mut mut_inner_cur = mut_cur;

                                while mut_inner_cur < events.len() {
                                    let keep_going = match &events[mut_inner_cur] {
                                        Event::Start(Tag::Heading { level, .. }) => {
                                            // keep growing
                                            *level != HeadingLevel::H1 && *level != HeadingLevel::H2
                                        }
                                        _ => true,
                                    };

                                    if keep_going {
                                        mut_inner_cur += 1;
                                    } else {
                                        break;
                                    }
                                }

                                if mut_inner_cur - mut_cur > 0 {
                                    mut_grouped_events
                                        .push(Grouped::Content(&events[mut_cur..mut_inner_cur]));
                                    mut_cur = mut_inner_cur;
                                } else {
                                    // We should not get here. This should've been handled by H1 or H2.
                                    log::error!(
                                        "Could not get any content within H2: {:?}",
                                        &events[mut_cur..mut_cur + min(5, events.len())]
                                    );

                                    // mut_cur += 1;
                                    panic!("Remove those!");
                                }
                            } else {
                                mut_cur += 1;
                            }
                        }
                    }
                }
            }
        }

        mut_grouped_events
    };

    // Skip all H1 content and what's under it and only keep relevant ones (old format events)
    let relevant_grouped_events = {
        let mut mut_relevant_grouped_events = vec![];
        let mut mut_h1_is_relevant = false;

        grouped_events.iter().for_each(|grp| {
            match grp {
                Grouped::H1(heading1) => {
                    if !OLD_FORMAT_HEADINGS.contains(&strip_autonumbered_sections(heading1).trim())
                    {
                        // Not relevant, not an old format heading
                        mut_h1_is_relevant = false;
                    } else {
                        mut_h1_is_relevant = true;
                        mut_relevant_grouped_events.push(grp);
                    }
                }
                _ => {
                    if !mut_h1_is_relevant {
                        // skip
                    } else {
                        mut_relevant_grouped_events.push(grp);
                    }
                }
            }
        });

        mut_relevant_grouped_events
    };

    // Now the old entry content are the Content groups, their name is in H2, and their category in H1.
    let old_format_entries = {
        let mut mut_old_format_entries = vec![];
        let mut mut_opt_last_entry_type: Option<OldFormatEntryType> = None;
        let mut mut_opt_last_entry_name: Option<String> = None;

        for grp in relevant_grouped_events {
            match grp {
                Grouped::H1(heading1) => {
                    let entry_type =
                        OldFormatEntryType::from_str(strip_autonumbered_sections(heading1).trim())
                            .map_err(GetNoteOldFormatEntriesError::InvalidEntryType)?;

                    mut_opt_last_entry_type = Some(entry_type)
                }
                Grouped::H2(heading2) => {
                    mut_opt_last_entry_name = Some(heading2.clone());
                }
                Grouped::Content(events) => {
                    let entry_type = mut_opt_last_entry_type.clone().pipe(|opt| match opt {
                        Some(val) => Ok(val),
                        None => Err(GetNoteOldFormatEntriesError::EventTypeAndNameNotConfigured),
                    })?;

                    let entry_name = mut_opt_last_entry_name.clone().pipe(|opt| match opt {
                        Some(val) => Ok(val),
                        None => Err(GetNoteOldFormatEntriesError::EventTypeAndNameNotConfigured),
                    })?;

                    mut_old_format_entries.push(OldFormatEntry {
                        entry_type,
                        entry_name,
                        events: events.to_vec(),
                    })
                }
            }
        }

        mut_old_format_entries
    };

    Ok(old_format_entries)
}

#[derive(Debug)]
pub enum SpawnMetadata<'a> {
    Spawning {
        event: Event<'a>,
        note_link: ObsidianLink,
        block_identifier: BlockIdentifier,
    },

    Spawned {
        event: Event<'a>,
        note_link: ObsidianLink,
        block_identifier: BlockIdentifier,
    },
}

impl<'a> GetEventText for SpawnMetadata<'a> {
    fn get_event_text(&self) -> Result<String, GetEventTextInternalError> {
        match self {
            SpawnMetadata::Spawning { event, .. } | SpawnMetadata::Spawned { event, .. } => {
                match event.clone() {
                    Event::Text(cow_str) => Ok(cow_str.to_string()),
                    _ => Err(GetEventTextInternalError::EventMustBeText),
                }
            }
        }
    }
}

/// This method fails on detecting patterns that should have been fixed manually.
pub fn extract_spawn_metadata_from_old_format<'a>(
    linkables: &'a [ObsidianLinkableItem<'a>],
    links: &'a [ObsidianLinkItem<'a>],
) -> Vec<SpawnMetadata<'a>> {
    // Spawn {note} ^spawn-{category}-{randhex6}
    let shared_event_items = linkables
        .iter()
        .cartesian_product(links.iter())
        .filter(|(linkable, link)| linkable.event == link.event)
        .collect::<Vec<_>>();

    let spawns = shared_event_items
        .iter()
        .filter(|(linkable, _)| match &linkable.item_data {
            comm::ObsidianLinkableData::Heading(_, _) => false,
            comm::ObsidianLinkableData::BlockIdentifier(block_identifier) => {
                block_identifier.text.starts_with("^spawn")
            }
        })
        .filter(|(linkable, _)| match linkable.get_event_text() {
            Ok(text) => text.trim().starts_with("Spawn "),
            Err(_) => false,
        })
        .filter(|(_, link)| link.links.len() == 1)
        .flat_map(|(linkable, link)| match &linkable.item_data {
            comm::ObsidianLinkableData::Heading(_, _) => None,
            comm::ObsidianLinkableData::BlockIdentifier(block_identifier) => {
                Some(SpawnMetadata::Spawning {
                    event: link.event.clone(),
                    note_link: link.links[0].clone(),
                    block_identifier: block_identifier.clone(),
                })
            }
        })
        .collect::<Vec<_>>();

    // From [[#^spawn-{category}-{randhex6}]] in {note}
    let spawned = links
        .iter()
        .filter(|link| {
            link.links.len() == 2
                && match link.get_event_text() {
                    Ok(text) => text.starts_with("From ") && text.contains("in"),
                    Err(_) => false,
                }
        })
        .filter(|link| link.links[0].text.contains("spawn"))
        .flat_map(|link| {
            if let Some(sublink_with_hash) = link.links[0].opt_sublink.clone() {
                let sublink = sublink_with_hash.replace("#", "");

                if let Ok(block_identifier) = BlockIdentifier::from_str(&sublink) {
                    Some(SpawnMetadata::Spawned {
                        event: link.event.clone(),
                        note_link: link.links[1].clone(),
                        block_identifier,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Merge the results
    {
        let mut mut_merged = vec![];

        mut_merged.extend(spawns);
        mut_merged.extend(spawned);

        mut_merged
    }
}

pub fn remove_old_format_entries_from_note<'a>(
    _path: &Path,
    _events: &[&'a Event],
    _old_format_entries: &[OldFormatEntry<'a>],
) -> Option<()> {
    todo!()
}

pub fn turn_note_into_cluster_note(_path: &Path) -> Option<()> {
    todo!()
}

pub fn create_new_peripheral_note_from_old_format_entry<'a>(
    _root: ClusterRootFolderPath,
    _entry: OldFormatEntry<'a>,
) {
    todo!()
}

pub fn generate_index_for_core_note(_core_note: CoreNoteFilePath) -> Option<()> {
    todo!()
}

pub fn redirect_links_to_new_peripheral_note(
    _vault: &[WorkingPath],
    _opt_parent_link: Option<String>,
    _old_link: String,
    _new_link: String,
) -> Option<()> {
    // Include Timeline strings too
    todo!()
}
