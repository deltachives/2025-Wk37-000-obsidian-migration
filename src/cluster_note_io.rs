use pulldown_cmark::Event;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::cluster_note::*;

pub fn remove_old_format_entries_from_note<'a>(
    _path: &Path,
    _events: &[&'a Event],
    _old_format_entries: &[OldFormatEntry<'a>],
) -> Option<()> {
    todo!()
}

#[derive(Error, Debug)]
pub enum TurnNoteIntoClusterNoteAssertionError {
    #[error("Expecting all note files to have a parent folder! {0:?}")]
    NoParentFound(PathBuf),

    #[error("Failed to perform a cluster root folder check on {0:?}")]
    FailedClusterRootFolderCheck(PathBuf),
}

#[derive(Error, Debug)]
pub enum TurnNoteIntoClusterNoteError {
    #[error("An assumption broke: {0:?}")]
    AssertionFailed(#[from] TurnNoteIntoClusterNoteAssertionError),

    #[error("Provided path is not a file: {0:?}")]
    NotAFile(PathBuf),

    #[error("Provided path is already in a cluster root folder: {0:?}")]
    AlreadyInCluster(PathBuf),

    #[error("Failed to convert into a core note file path: {0:?}")]
    CoreNoteFilePathError(PathBuf),

    #[error("IO Check fails for {0:?}")]
    IoNone(PathBuf),

    #[error("Got IO Error {0:?}")]
    Io(#[from] std::io::Error),
}

pub fn turn_note_into_cluster_note(
    path: &Path,
) -> Result<CoreNoteFilePath, TurnNoteIntoClusterNoteError> {
    if !path.is_file() {
        return Err(TurnNoteIntoClusterNoteError::NotAFile(path.to_owned()));
    }

    let parent = path
        .parent()
        .ok_or(TurnNoteIntoClusterNoteError::AssertionFailed(
            TurnNoteIntoClusterNoteAssertionError::NoParentFound(path.to_owned()),
        ))?;

    {
        let in_cluster_root_folder =
            is_cluster_root_folder(parent).ok_or(TurnNoteIntoClusterNoteError::AssertionFailed(
                TurnNoteIntoClusterNoteAssertionError::FailedClusterRootFolderCheck(
                    parent.to_owned(),
                ),
            ))?;

        if in_cluster_root_folder {
            return Err(TurnNoteIntoClusterNoteError::AlreadyInCluster(
                parent.to_owned(),
            ));
        }
    }

    // create a new folder with the same name as the note in parent

    let stem = path
        .file_stem()
        .ok_or(TurnNoteIntoClusterNoteError::IoNone(path.to_owned()))?;

    let new_cluster_root_folder = {
        let mut mut_new_cluster_root_folder = parent.to_path_buf();

        mut_new_cluster_root_folder.push(stem);

        mut_new_cluster_root_folder
    };

    fs::create_dir(&new_cluster_root_folder)?;

    // Copy the note over to there

    let new_core_note_path = {
        let mut mut_new_core_note_path = new_cluster_root_folder.clone();

        let file_name = path
            .file_name()
            .ok_or(TurnNoteIntoClusterNoteError::IoNone(path.to_path_buf()))?;

        mut_new_core_note_path.push(file_name);

        mut_new_core_note_path
    };

    fs::copy(path, &new_core_note_path)?;

    // Delete the note

    fs::remove_file(path)?;

    let out = CoreNoteFilePath::new(&new_core_note_path).ok_or(
        TurnNoteIntoClusterNoteError::CoreNoteFilePathError(new_core_note_path),
    )?;

    Ok(out)
}

#[derive(Error, Debug)]
pub enum CreateNewPeripheralNoteFromOldFormatEntryError {
    #[error("Got IO Error {0:?}")]
    Io(PathBuf, std::io::Error),

    #[error("Failed to get and categories directory entries: {0:?}")]
    GetAndCategorizeDirEntriesError(#[from] crate::common::GetAndCategorizeDirEntriesError),

    #[error("Category path must have only files, not {0:?}")]
    DirEntyMustBeFile(crate::common::CategorizedDirEntry),
}

pub fn create_new_peripheral_note_from_old_format_entry(
    cluster_folder_path: &ClusterFolderPath,
    entry: &OldFormatEntryContent,
) -> Result<(), CreateNewPeripheralNoteFromOldFormatEntryError> {
    type FnErr = CreateNewPeripheralNoteFromOldFormatEntryError;

    let category_path = {
        let mut mut_category_path = cluster_folder_path.path.clone();

        mut_category_path.push(entry.entry_type.to_context_type_folder());

        mut_category_path
    };

    // Create the category folder if it doesn't exist

    fs::create_dir_all(&category_path).map_err(|e| FnErr::Io(category_path.clone(), e))?;

    // Count how many files exist there to determine the file name
    let num_entries = {
        let mut mut_num_entries: usize = 0;

        let dir_entries = crate::common::get_and_categorize_dir_entries(&category_path)?;

        for dir_entry in dir_entries {
            match dir_entry {
                crate::common::CategorizedDirEntry::File(_) => mut_num_entries += 1,
                _ => return Err(FnErr::DirEntyMustBeFile(dir_entry)),
            }
        }

        mut_num_entries
    };

    let num_entries_s = {
        if num_entries < 10 {
            format!("00{num_entries}")
        } else if num_entries < 100 {
            format!("0{num_entries}")
        } else {
            num_entries.to_string()
        }
    };

    let peripheral_note_path = {
        let mut mut_peripheral_note_path = category_path;

        mut_peripheral_note_path.push(format!("{num_entries_s} {}.md", entry.entry_name));

        mut_peripheral_note_path
    };

    crate::common::write_file_content(
        &format!("\n# 1 Journal\n\n{}", entry.content),
        &peripheral_note_path,
    )
    .map_err(|e| FnErr::Io(peripheral_note_path.clone(), e))?;

    Ok(())
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
