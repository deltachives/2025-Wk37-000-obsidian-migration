use itertools::Itertools;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd, TextMergeStream};
use pulldown_cmark_to_cmark::cmark_with_options;
use std::{
    fs::{DirEntry, File},
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use tap::prelude::*;
use thiserror::Error;

#[derive(Debug)]
pub enum CategorizedDirEntry {
    Dir(DirEntry),
    File(DirEntry),
    Symlink(DirEntry),
}

#[derive(Error, Debug)]
pub enum GetAndCategorizeDirEntriesAssertError {
    #[error("Unrecognized metadata category for {0:?}")]
    UnrecognizedMetadaCategory(PathBuf),
}

#[derive(Error, Debug)]
pub enum GetAndCategorizeDirEntriesError {
    #[error("Failed io operation: {0:?}")]
    Io(#[from] std::io::Error),

    #[error("Assert error: {0:?}")]
    AssertError(#[from] GetAndCategorizeDirEntriesAssertError),
}

pub fn get_and_categorize_dir_entries(
    path: &Path,
) -> Result<Vec<CategorizedDirEntry>, GetAndCategorizeDirEntriesError> {
    let dir = path.read_dir()?;

    let dir_entries = dir.collect::<Result<Vec<_>, _>>()?;

    let categorized = dir_entries
        .into_iter()
        .map(|entry| {
            let metadata = entry.metadata()?;

            let res_categorized: Result<CategorizedDirEntry, GetAndCategorizeDirEntriesError> = {
                if metadata.is_dir() {
                    Ok(CategorizedDirEntry::Dir(entry))
                } else if metadata.is_file() {
                    Ok(CategorizedDirEntry::File(entry))
                } else if metadata.is_symlink() {
                    Ok(CategorizedDirEntry::Symlink(entry))
                } else {
                    Err(GetAndCategorizeDirEntriesError::AssertError(
                        GetAndCategorizeDirEntriesAssertError::UnrecognizedMetadaCategory(
                            entry.path(),
                        ),
                    ))
                }
            };

            res_categorized
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(categorized)
}

pub fn get_folder_child_file_count_non_recursive(folder: &Path) -> Option<usize> {
    let dir_entries = get_and_categorize_dir_entries(folder).ok()?;

    let count = dir_entries
        .into_iter()
        .filter(|e| matches!(e, CategorizedDirEntry::File(..)))
        .count();

    Some(count)
}

/// Checks that the folder has a direct child file with the same name (ignoring extensions).
/// Returns None on error.
pub fn folder_has_file_of_same_name(folder: &Path) -> Option<bool> {
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

pub fn read_file_content(path: &Path) -> Option<String> {
    let content = {
        let mut file = File::open(path).ok()?;

        let mut out = String::new();

        file.read_to_string(&mut out).ok()?;

        out
    };

    Some(content)
}

pub fn write_file_content(s: &str, output_path: &Path) -> std::io::Result<usize> {
    let mut file = std::fs::File::create(output_path)?;

    std::io::Write::write(&mut file, s.as_bytes())
}

#[derive(Error, Debug)]
pub enum RenderEventsToCommonMarkdownError {
    #[error("Failed to convert events back to cmark: {0:?}")]
    PulldownCmarkToCmarkError(#[from] pulldown_cmark_to_cmark::Error),
}

pub fn render_events_to_common_markdown<'a>(
    events: &'a [Event<'a>],
) -> Result<String, RenderEventsToCommonMarkdownError> {
    let mut mut_out = String::new();

    let _ = cmark_with_options(
        events.iter(),
        &mut mut_out,
        pulldown_cmark_to_cmark::Options {
            newlines_after_headline: 2,
            newlines_after_paragraph: 2,
            newlines_after_codeblock: 2,
            newlines_after_htmlblock: 1,
            newlines_after_table: 2,
            newlines_after_rule: 2,
            newlines_after_list: 2,
            newlines_after_blockquote: 2,
            newlines_after_rest: 1,
            newlines_after_metadata: 1,
            code_block_token_count: 3,
            code_block_token: '`',
            list_token: '-',
            ordered_list_token: '.',
            increment_ordered_list_bullets: true,
            emphasis_token: '*',
            strong_token: "**",
        },
    )?;

    Ok(mut_out)
}

/// Applies some fixes to rendered markdown files to be obsidian compliant. This is quite adhoc and
/// is likely not exhaustive.
pub fn adhoc_fix_rendered_markdown_output_for_obsidian(
    old_content: &str,
    new_content: &str,
) -> String {
    fn log_(_s: &str) {
        // log::trace!("{}", s)
    }

    let new_content1 = {
        let mut mut_out = String::new();

        log_(&format!("<new_content>\n{new_content}\n</new_content>"));

        let diff = similar::TextDiff::from_words(old_content, new_content);

        for change in diff.iter_all_changes() {
            let s = change.value();

            match change.tag() {
                similar::ChangeTag::Equal => mut_out += s,
                similar::ChangeTag::Delete => {
                    // In general, we don't want to be adding additions from old content. Some exceptions:
                    // - Obsidian sometimes adds escaping. For example for obsidian link title bars in tables.
                    // - Keep obsidian frontmatter "---" intact
                    // - Keep `_` which may be added in math

                    if s.trim() == "\\" || s.trim() == "_" {
                        log_(&format!("-0.0 \"{s}\""));
                        mut_out += s;
                    } else if s.trim() == "---" {
                        log_(&format!("-0.1 \"{s}\""));
                        mut_out += "\n---"; // would be on same line as last prop without new line
                    } else {
                        log_(&format!("-0.2 \"{s}\""));
                    }
                }
                similar::ChangeTag::Insert => {
                    // In general, we want to add additions from new content. Some exceptions:
                    // - They add unnecessary escaping
                    // - They mess up frontmatter by adding `##` for the first property
                    // - In some instances, math is changes by removing `_` for `*`.
                    // - In case of subtask - [ ] , they may change them to - \[ ] with "- \\". This needs to be modified.
                    //   Note that this is for the setting of using "-" bullets for markdown rendering.

                    if s.trim() != "\\"
                        && s.trim() != ""
                        && s.trim() != "##"
                        && s.trim() != "- \\"
                        && s.trim() != "\\|"
                    {
                        log_(&format!("+0.0 \"{s}\""));
                        mut_out += s;
                    } else if s.trim() == "- \\" || s.trim() == "\\|" {
                        log_(&format!("+0.1 \"{s}\""));
                        mut_out += &s.replace("\\", "");
                    } else if s.trim() == "" && s.contains("\n") {
                        log_(&format!("+0.2 \"{s}\""));
                        mut_out += "\n";
                    } else {
                        log_(&format!("+0.3 \"{s}\""));
                    }
                }
            }
        }

        mut_out
    };

    // Now we run a character diff processing on some remaining items
    {
        let mut mut_out = String::new();

        log_(&format!("<new_content1>\n{new_content1}\n</new_content1>"));

        let diff = similar::TextDiff::from_chars(old_content, &new_content1);

        for change in diff.iter_all_changes() {
            let s = change.value();

            match change.tag() {
                similar::ChangeTag::Equal => mut_out += s,
                similar::ChangeTag::Delete => {
                    // Remaining additions from old content include:
                    // - Obsidian sometimes adds escaping. For example for obsidian link title bars in tables.

                    if s.trim() == "\\" || s.trim() == "_" {
                        log_(&format!("-1.0 \"{s}\""));
                        mut_out += s;
                    } else {
                        log_(&format!("-1.S \"{s}\""));
                    }
                }
                similar::ChangeTag::Insert => {
                    // Remaining additions from new content include:
                    // - Removing escaped bars from obsidian links within tables (tested by: test_obsidian_patch_writeback table-002)
                    // - extra quote ">" lines are added and they shouldn't be
                    // - Sometimes when * bullets are replaced for dashes, a space is not inserted.

                    if s.trim() != "\\"
                        && s.trim() != ""
                        && s.trim() != "*"
                        && s.trim() != ">"
                        && s.trim() != "-"
                    {
                        log_(&format!("+1.0 \"{s}\""));
                        mut_out += s;
                    } else if s.trim() == "-" {
                        log_(&format!("+1.1 \"{s}\""));
                        mut_out += "- ";
                    } else {
                        log_(&format!("+1.S \"{s}\""));
                    }
                }
            }
        }

        mut_out
    }
}

pub fn parse_markdown_file<'a>(content: &'a str) -> Vec<Event<'a>> {
    let parser = Parser::new(content);

    TextMergeStream::new(parser).collect_vec()
}

pub fn parse_markdown_file_frontmatter_section(path: &Path) -> Option<Vec<(String, String)>> {
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
                Event::End(pulldown_cmark::TagEnd::Heading(heading_level)) => {
                    if *heading_level != HeadingLevel::H2 {
                        break;
                    }

                    // Processed everything
                    mut_broke_format = false;
                    break;
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

pub fn get_file_frontmatter_note_property(path: &Path, prop: &str) -> Option<String> {
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

pub fn is_obsidian_vault_folder(path: &Path) -> Option<bool> {
    let dir_entries = get_and_categorize_dir_entries(path).ok()?;

    dir_entries
        .into_iter()
        .filter(|entry| match entry {
            CategorizedDirEntry::Dir(dir_entry) => dir_entry.file_name() == ".obsidian",
            _ => false,
        })
        .count()
        .pipe(|count| Some(count == 1))
}

#[derive(Debug)]
pub struct ObsidianVaultPath {
    pub path: PathBuf,
}

impl ObsidianVaultPath {
    pub fn new(path: &Path) -> Option<Self> {
        if !is_obsidian_vault_folder(path)? {
            return None;
        }

        Some(Self {
            path: path.to_owned(),
        })
    }
}

pub fn process_heading_event<'a>(events: &[Event<'a>]) -> Option<(HeadingLevel, String)> {
    // Process H{level}, then Text with the tring, then /H{level}.
    if events.len() < 3 {
        return None;
    }

    let level = match &events[0] {
        Event::Start(Tag::Heading { level, .. }) => Some(level),
        _ => None,
    }?;

    let heading_text = match &events[1] {
        Event::Text(cow_str) => Some(cow_str.to_string()),
        _ => None,
    }?;

    match &events[2] {
        Event::End(TagEnd::Heading(heading_level)) => {
            if heading_level != level {
                log::error!("Assertion failed: Data does not properly close same heading level");
            }
        }
        _ => return None,
    }

    Some((*level, heading_text))
}

#[derive(Error, Debug)]
pub enum ProcessHeadingEventInternalError {
    #[error("It is expected that a heading tag start and end have identical levels")]
    ImbalancedHeadingLevels,
}

#[derive(Error, Debug)]
pub enum ProcessHeadingEventError {
    #[error("Heading events come in 3, and there aren't enough events")]
    RequiresThreeEvents,

    #[error(
        "Invalid structure. We expect a heading start, then text, then heading end. But on the {0}th we get: {1}."
    )]
    InvalidScheme(u32, String),

    #[error("Expected a heading start tag but got {0:?}")]
    InvalidStartTag(String),

    #[error("Expected a heading end tag but got {0:?}")]
    InvalidEndTag(String),

    #[error("Expected heading level {0:?} but got {1:?}")]
    WrongLevel(HeadingLevel, HeadingLevel),

    #[error("Unexpected error occured")]
    Internal(#[from] ProcessHeadingEventInternalError),
}

pub fn process_heading_event_of_level<'a>(
    level: &HeadingLevel,
    events: &[Event<'a>],
) -> Result<String, ProcessHeadingEventError> {
    // Process H{level}, then Text with the tring, then /H{level}.
    if events.len() < 3 {
        return Err(ProcessHeadingEventError::RequiresThreeEvents);
    }

    match &events[0] {
        Event::Start(tag) => match tag {
            Tag::Heading {
                level: heading_level,
                ..
            } => {
                if heading_level != level {
                    return Err(ProcessHeadingEventError::WrongLevel(*level, *heading_level));
                }
            }
            _ => {
                return Err(ProcessHeadingEventError::InvalidStartTag(format!(
                    "{tag:?}"
                )));
            }
        },
        _ => {
            return Err(ProcessHeadingEventError::InvalidScheme(
                0,
                format!("{:?}", events[0]),
            ));
        }
    }

    let heading_text = match &events[1] {
        Event::Text(cow_str) => Ok(cow_str.to_string()),
        _ => Err(ProcessHeadingEventError::InvalidScheme(
            1,
            format!("{:?}", events[1]),
        )),
    }?;

    match &events[2] {
        Event::End(tag) => match tag {
            TagEnd::Heading(heading_level) => {
                if heading_level != level {
                    return Err(ProcessHeadingEventError::Internal(
                        ProcessHeadingEventInternalError::ImbalancedHeadingLevels,
                    ));
                }
            }
            _ => return Err(ProcessHeadingEventError::InvalidEndTag(format!("{tag:?}"))),
        },
        _ => {
            return Err(ProcessHeadingEventError::InvalidScheme(
                2,
                format!("{:?}", events[2]),
            ));
        }
    }

    Ok(heading_text)
}

#[derive(Debug, Clone)]
pub struct BlockIdentifier {
    pub text: String,
}

impl FromStr for BlockIdentifier {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('^') {
            return Err(());
        }

        let valid = &s[1..].chars().all(|c| c.is_alphanumeric() || c == '-');

        if !valid {
            return Err(());
        }

        Ok(Self {
            text: s.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum ObsidianLinkableData {
    Heading(HeadingLevel, String),
    BlockIdentifier(BlockIdentifier),
}

#[derive(Debug, Clone)]
pub struct ObsidianLinkableItem<'a> {
    pub item_data: ObsidianLinkableData,
    pub event: Event<'a>,
}

#[derive(Error, Debug)]
pub enum GetEventTextInternalError {
    #[error("Obsidian Text Item events must be Text events")]
    EventMustBeText,
}

pub trait GetEventText {
    fn get_event_text(&self) -> Result<String, GetEventTextInternalError>;
}

impl<'a> GetEventText for ObsidianLinkableItem<'a> {
    fn get_event_text(&self) -> Result<String, GetEventTextInternalError> {
        match self.event.clone() {
            Event::Text(cow_str) => Ok(cow_str.to_string()),
            _ => Err(GetEventTextInternalError::EventMustBeText),
        }
    }
}

pub fn extract_linkable_obsidian_md_items<'a>(
    events: &Vec<Event<'a>>,
) -> Vec<ObsidianLinkableItem<'a>> {
    let headings_items = (0..events.len())
        .map(|i| (events[i].clone(), process_heading_event(&events[i..])))
        .filter(|(_, heading_tup)| heading_tup.is_some())
        .map(|(event, heading_tup)| (event, heading_tup.expect("Nones are filtered out")))
        .map(|(event, (level, heading))| ObsidianLinkableItem {
            item_data: ObsidianLinkableData::Heading(level, heading),
            event,
        })
        .collect::<Vec<_>>();

    let block_identifier_items = events
        .iter()
        .flat_map(|event| match event {
            Event::Text(cow_str) => {
                let rev_caret_pos = cow_str.chars().rev().position(|c| c == '^')?;

                let len = cow_str.chars().count();

                let block_identifier = cow_str
                    .chars()
                    .skip(len - rev_caret_pos)
                    .join("")
                    .pipe(|s| BlockIdentifier::from_str(&s))
                    .ok()?;

                Some((event.clone(), block_identifier))
            }
            _ => None,
        })
        .map(|(event, block_identifier)| ObsidianLinkableItem {
            item_data: ObsidianLinkableData::BlockIdentifier(block_identifier),
            event,
        })
        .collect::<Vec<_>>();

    {
        let mut mut_merged = vec![];

        mut_merged.extend(headings_items);
        mut_merged.extend(block_identifier_items);

        mut_merged
    }
}

#[derive(Debug, Clone)]
pub struct ObsidianLink {
    pub text: String,
    pub opt_file_link: Option<String>,
    pub opt_sublink: Option<String>,
    pub opt_title: Option<String>,
}

#[derive(Error, Debug)]
pub enum ObsidianLinkParseError {
    #[error("An obsidian link must start with [[ and end with ]]: {0:?}")]
    NoBracketsFound(String),
    #[error("An obsidian link can only have one '#' denoting the sublink but found {0}: {1:?}")]
    MustHaveZeroOrOneHash(usize, String),
    #[error("An obsidian link can only have one '|' denoting the sublink but found {0}: {1:?}")]
    MustHaveZeroOrOneBar(usize, String),
}

impl FromStr for ObsidianLink {
    type Err = ObsidianLinkParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("[[") || !s.ends_with("]]") {
            return Err(ObsidianLinkParseError::NoBracketsFound(s.to_owned()));
        }

        let remaining_s = s.replace("[[", "").replace("]]", "");

        // There can only be one # and one |
        let count_hash = remaining_s.chars().filter(|c| *c == '#').count();

        let count_bar = remaining_s.chars().filter(|c| *c == '|').count();

        if count_hash > 1 {
            return Err(ObsidianLinkParseError::MustHaveZeroOrOneHash(
                count_hash,
                remaining_s.to_owned(),
            ));
        }

        if count_bar > 1 {
            return Err(ObsidianLinkParseError::MustHaveZeroOrOneBar(
                count_bar,
                remaining_s.to_owned(),
            ));
        }

        let opt_title_pos = remaining_s.chars().position(|c| c == '|');

        let opt_sublink_pos = remaining_s.chars().position(|c| c == '#');

        let opt_title = match opt_title_pos {
            Some(title_pos) => remaining_s.chars().skip(title_pos).join("").pipe(Some),
            None => None,
        };

        let opt_sublink = match opt_sublink_pos {
            Some(sublink_pos) => {
                let remaining_s2 = match opt_title.clone() {
                    Some(title) => remaining_s.replace(&title, ""),
                    None => remaining_s.clone(),
                };

                remaining_s2.chars().skip(sublink_pos).join("").pipe(Some)
            }
            None => None,
        };

        let remaining_s2 = {
            remaining_s
                .to_string()
                .pipe(|s| match opt_title.clone() {
                    Some(title) => s.replace(&title, ""),
                    None => s,
                })
                .pipe(|s| match opt_sublink.clone() {
                    Some(sublink) => s.replace(&sublink, ""),
                    None => s,
                })
        };

        // And whatever remains can only be the file link if it exists
        let opt_file_link = match remaining_s2.is_empty() {
            true => None,
            false => Some(remaining_s2),
        };

        Ok(Self {
            text: s.to_string(),
            opt_file_link,
            opt_sublink,
            opt_title,
        })
    }
}

pub fn parse_multiple_obsidian_links(s: &str) -> Result<Vec<ObsidianLink>, ObsidianLinkParseError> {
    let tokens = s
        .split("[[")
        .filter(|s| s.contains("]]"))
        .map(|s| {
            let end_idx = s
                .find("]]")
                .expect("Assertion failed: Already filtered for ]]");

            let stripped = s.chars().take(end_idx + "]]".len()).join("");

            format!("[[{stripped}")
        })
        .collect::<Vec<_>>();

    let links = tokens
        .iter()
        .map(|token| ObsidianLink::from_str(token))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(links)
}

#[derive(Debug, Clone)]
pub struct ObsidianLinkItem<'a> {
    pub links: Vec<ObsidianLink>,
    pub event: Event<'a>,
}

impl<'a> GetEventText for ObsidianLinkItem<'a> {
    fn get_event_text(&self) -> Result<String, GetEventTextInternalError> {
        match self.event.clone() {
            Event::Text(cow_str) => Ok(cow_str.to_string()),
            _ => Err(GetEventTextInternalError::EventMustBeText),
        }
    }
}

#[derive(Error, Debug)]
pub enum ExtractOBsidianMdLinksError {
    #[error("Failed to extract obsidian link: {0:?}")]
    LinkExtractError(#[from] ObsidianLinkParseError),
}

pub fn extract_obsidian_md_links<'a>(
    events: &Vec<Event<'a>>,
) -> Result<Vec<ObsidianLinkItem<'a>>, ExtractOBsidianMdLinksError> {
    let extracted = events
        .iter()
        .map(|event| match event {
            Event::Text(cow_str) => {
                let links = parse_multiple_obsidian_links(cow_str)
                    .map_err(ExtractOBsidianMdLinksError::LinkExtractError)?;

                if links.is_empty() {
                    return Ok(None);
                }

                Ok(Some((event.clone(), links)))
            }
            _ => Ok(None),
        })
        .filter_ok(|opt| opt.is_some())
        .map_ok(|opt| opt.expect("Nones are filtered out"))
        .map_ok(|(event, links)| ObsidianLinkItem { links, event })
        .collect::<Result<Vec<_>, ExtractOBsidianMdLinksError>>()?;

    Ok(extracted)
}
