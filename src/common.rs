use itertools::Itertools;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TextMergeStream};
use std::{
    fs::{DirEntry, File},
    io::Read,
    path::PathBuf,
};
use tap::prelude::*;

pub enum CategorizedDirEntry {
    Dir(DirEntry),
    File(DirEntry),
    Symlink(DirEntry),
}

pub fn get_and_categorize_dir_entries<'a>(
    path: &PathBuf,
) -> Result<Vec<CategorizedDirEntry>, String> {
    let dir = path.read_dir().map_err(|e| e.to_string())?;

    let dir_entries = dir
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let categorized = dir_entries
        .into_iter()
        .map(|entry| {
            let metadata = entry.metadata().map_err(|e| e.to_string())?;

            let res_categorized: Result<CategorizedDirEntry, String> = {
                if metadata.is_dir() {
                    Ok(CategorizedDirEntry::Dir(entry))
                } else if metadata.is_file() {
                    Ok(CategorizedDirEntry::File(entry))
                } else if metadata.is_symlink() {
                    Ok(CategorizedDirEntry::Symlink(entry))
                } else {
                    Err(format!(
                        "Assertion failed: Unrecognized metadata category for {:?}",
                        entry.path()
                    ))
                }
            };

            res_categorized
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(categorized)
}

pub fn get_folder_child_file_count_non_recursive(folder: &PathBuf) -> Option<usize> {
    let dir_entries = get_and_categorize_dir_entries(folder).ok()?;

    let count = dir_entries
        .into_iter()
        .filter(|e| match e {
            CategorizedDirEntry::File(_) => true,
            _ => false,
        })
        .count();

    Some(count)
}

/// Checks that the folder has a direct child file with the same name (ignoring extensions).
/// Returns None on error.
pub fn folder_has_file_of_same_name(folder: &PathBuf) -> Option<bool> {
    let dir_entries = get_and_categorize_dir_entries(folder).ok()?;

    let folder_name = folder.file_name()?;

    let found = dir_entries
        .into_iter()
        .filter(|entry| match entry {
            CategorizedDirEntry::File(dir_entry) => match dir_entry.path().file_stem() {
                Some(filename) => folder_name == filename,
                None => false,
            },
            _ => false,
        })
        .count();

    Some(found == 1)
}

pub fn read_file_content(path: &PathBuf) -> Option<String> {
    let content = {
        let mut file = File::open(&path).ok()?;

        let mut out = String::new();

        file.read_to_string(&mut out).ok()?;

        out
    };

    Some(content)
}

pub fn parse_markdown_file<'a>(content: &'a str) -> Vec<Event<'a>> {
    let parser = Parser::new(&content);

    let result = TextMergeStream::new(parser).collect_vec();

    result
}

pub fn parse_markdown_file_frontmatter_section(path: &PathBuf) -> Option<Vec<(String, String)>> {
    let content = read_file_content(path)?;

    let events = parse_markdown_file(&content);

    // We need to parse events "Rule H2 (Text [Softbreak] ...) /H2"

    if events.len() < 3 {
        return None;
    }

    if !matches!(events[0], Event::Rule) {
        return None;
    }

    if !matches!(
        events[1],
        Event::Start(Tag::Heading {
            level: HeadingLevel::H2,
            ..
        })
    ) {
        return None;
    }

    // Now parse either Text or SoftBreak, until we hit /H2 or EOF.
    let props = {
        let mut mut_props: Vec<(String, String)> = vec![];
        let mut mut_broke_format: bool = true;

        for event in &events[2..] {
            match event {
                Event::End(tag_end) => {
                    match tag_end {
                        pulldown_cmark::TagEnd::Heading(heading_level) => {
                            if *heading_level != HeadingLevel::H2 {
                                break;
                            }

                            // Processed everything
                            mut_broke_format = false;
                            break;
                        }
                        _ => break,
                    }
                }
                Event::Text(cow_str) => {
                    let split = cow_str.split(":").collect_vec();

                    if split.len() != 2 {
                        break;
                    }

                    mut_props.push((split[0].to_owned(), split[1].to_owned()))
                }
                Event::SoftBreak => continue,
                _ => break,
            }
        }

        if mut_broke_format {
            None
        } else {
            Some(mut_props)
        }
    }?;

    Some(props)
}

pub fn get_file_frontmatter_note_property(path: &PathBuf, prop: &str) -> Option<String> {
    let props = parse_markdown_file_frontmatter_section(path)?;

    let matching_value = props
        .into_iter()
        .find(|(k, _)| prop == k)?
        .1
        .trim()
        .to_owned();

    // we need to clean up the note value. We expect it to look like [[this]], so remove those boxes!
    if !matching_value.starts_with("[[") || !matching_value.ends_with("]]") {
        return None;
    }

    let matching_note_link = matching_value.replace("[[", "").replace("]]", "");

    Some(matching_note_link)
}


pub fn is_obsidian_vault_folder(path: &PathBuf) -> Option<bool> {
    let dir_entries = get_and_categorize_dir_entries(path).ok()?;

    dir_entries
        .into_iter()
        .filter(|entry| {
            match entry {
                CategorizedDirEntry::Dir(dir_entry) => {
                    dir_entry.file_name() == ".obsidian"
                },
                _ => false,
            }
        })
        .count()
        .pipe(|count| Some(count == 1))
}

#[derive(Debug)]
pub struct ObsidianVaultPath {
    pub path: PathBuf,
}

impl ObsidianVaultPath {
    pub fn new(path: &PathBuf) -> Option<Self> {
        if !is_obsidian_vault_folder(path)? {
            return None;
        }

        Some(Self { path: path.clone() })
    }
}