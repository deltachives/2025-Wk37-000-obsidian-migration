use std::path::PathBuf;

use crate::common::{self as comm, CategorizedDirEntry};

use pulldown_cmark::Event;
use tap::prelude::*;

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

pub const _OLD_FORMAT_HEADINGS: [&str; _NUM_OLD_FORMAT_HEADINGS] = [
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
        .find(|heading| *heading == CONTEXT_TYPE_HEADINGS[context_type_id])
        .is_some()
}

pub fn file_exists_in_folder_of_same_name(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }

    let inner_fn = move || {
        let parent_folder = path.parent()?;

        let file_stem = path.file_stem()?;

        Some(path.is_file() && path.exists() && parent_folder.ends_with(file_stem))
    };

    match inner_fn() {
        Some(res) => res,
        None => false,
    }
}

pub fn is_cluster_root_folder(folder: &PathBuf) -> Option<bool> {
    if !comm::folder_has_file_of_same_name(folder)? {
        return Some(false);
    }

    if comm::get_folder_child_file_count_non_recursive(folder)? != 1 {
        return Some(false);
    }

    Some(true)
}

pub fn is_cluster_category_folder(folder: &PathBuf) -> Option<bool> {
    let folder_name = folder.file_name()?.to_str()?;

    if !CONTEXT_TYPE_FOLDERS.contains(&folder_name) {
        return Some(false);
    }

    if !is_cluster_root_folder(&folder.parent()?.to_path_buf())? {
        return Some(false);
    }

    Some(true)
}

pub fn is_markdown_file_path(path: &PathBuf) -> bool {
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

pub fn is_cluster_core_file_path(path: &PathBuf) -> Option<bool> {
    if !is_markdown_file_path(path) {
        return Some(false);
    }

    let parent = path.parent()?.to_path_buf();

    if !is_cluster_root_folder(&parent)? {
        return Some(false);
    }

    Some(true)
}

pub fn is_cluster_peripheral_file_path(path: &PathBuf) -> Option<bool> {
    if !is_markdown_file_path(path) {
        return Some(false);
    }

    let parent = path.parent()?.to_path_buf();

    if !is_cluster_category_folder(&parent)? {
        return Some(false);
    }

    Some(true)
}

pub fn is_normal_markdown_file_path(path: &PathBuf) -> Option<bool> {
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
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_cluster_root_folder(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}

#[derive(Debug)]
pub struct ClusterCategoryFolderPath {
    pub path: PathBuf,
}

impl ClusterCategoryFolderPath {
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_cluster_category_folder(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}

#[derive(Debug)]
pub struct CoreNoteFilePath {
    pub path: PathBuf,
}

impl CoreNoteFilePath {
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_cluster_core_file_path(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}

#[derive(Debug)]
pub struct PeripheralNoteFilePath {
    pub path: PathBuf,
}

impl PeripheralNoteFilePath {
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_cluster_peripheral_file_path(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}

#[derive(Debug)]
pub struct NormalNoteFilePath {
    pub path: PathBuf,
}

impl NormalNoteFilePath {
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_normal_markdown_file_path(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}

pub fn get_core_note_file_from_cluster_root_folder(
    cluster_root_folder: &ClusterRootFolderPath,
) -> Option<CoreNoteFilePath> {
    let dir_entries = comm::get_and_categorize_dir_entries(&cluster_root_folder.path).ok()?;

    let core_note_path = dir_entries
        .into_iter()
        .filter(|entry| matches!(entry, CategorizedDirEntry::File(_)))
        .next()?
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

pub fn get_working_item_paths_recursive(folder: &PathBuf) -> Option<Vec<WorkingPath>> {
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

pub fn note_link_to_path(vault: &Vec<WorkingPath>, note_link: &str) -> Option<PathBuf> {
    let path = vault
        .iter()
        .map(|item| match item {
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
                        .map(|file| {
                            if file.path.ends_with(note_link) {
                                Some(file.path.clone())
                            } else {
                                None
                            }
                        })
                        .filter(|opt_path| opt_path.is_some())
                        .map(|opt_path| opt_path.expect("Nones should already be filtered out"))
                        .next()
                }
            }
        })
        .filter(|opt_path| opt_path.is_some())
        .map(|opt_path| opt_path.expect("Nones should already be filtered out"))
        .next()?;

    Some(path)
}

pub fn get_cluster_core_file_from_peripheral(
    vault: &Vec<WorkingPath>,
    peripheral_file: &PeripheralNoteFilePath,
) -> Option<CoreNoteFilePath> {
    let parent_note_link =
        comm::get_file_frontmatter_note_property(&peripheral_file.path, "parent")?;

    let path = note_link_to_path(vault, &parent_note_link)?;

    CoreNoteFilePath::new(&path)
}

pub enum OldFormatEntryType {
    Task,
    Issue,
    HowTo,
    Investigation,
    Idea,
    SideNote,
}

pub struct OldFormatEntry<'a> {
    pub entry_name: String,
    pub entry_type: OldFormatEntryType,
    pub events: Vec<Event<'a>>,
}

pub fn get_note_old_format_entries<'a>(_events: &Vec<&'a Event>) -> Vec<OldFormatEntry<'a>> {
    todo!()
}

pub fn remove_old_format_entries_from_note<'a>(
    _path: &PathBuf,
    _events: &Vec<&'a Event>,
    _old_format_entries: &Vec<OldFormatEntry<'a>>,
) -> Option<()> {
    todo!()
}

pub fn turn_note_into_cluster_note(_path: &PathBuf) -> Option<()> {

    todo!()
}

pub fn create_new_peripheral_note_from_old_format_entry<'a>(_root: ClusterRootFolderPath, _entry: OldFormatEntry<'a>) {
    todo!()
}

pub fn generate_index_for_core_note(_core_note: CoreNoteFilePath) -> Option<()> {
    todo!()
}

pub fn redirect_links_to_new_peripheral_note(_vault: &Vec<WorkingPath>, _opt_parent_link: Option<String>, _old_link: String, _new_link: String) -> Option<()> {
    // Include Timeline strings too
    todo!()
}
