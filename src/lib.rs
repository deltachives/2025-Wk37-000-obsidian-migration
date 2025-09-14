use lan_rs_common::util::io;
use std::path::PathBuf;

pub mod common;
pub mod cluster_note;

pub trait FromRepo {
    type Out;

    fn from_repo(path: &PathBuf) -> Vec<Self::Out>;
}

pub enum NoteClass {
    /// Note is in a folder named after it with folder structure
    BigNote,
    SmalNote,
    HybridNote,
    SmallHybridNote,
}

pub trait ClassifyByNoteClass {
    type Out;
    type Err;

    fn classify_by_note_class(&self) -> Result<(Self::Out, NoteClass), Self::Err>;
}

pub struct MarkdownFileData {
    _path: PathBuf,
    _content: String,
}

impl FromRepo for MarkdownFileData {
    type Out = MarkdownFileData;

    fn from_repo(path: &PathBuf) -> Vec<Self::Out> {
        let globbed = io::glob_multiple_file_formats_in_path(path, &["md"]);

        let content = io::read_file_to_string(&path.clone().to_string_lossy());

        globbed
            .into_iter()
            .map(|path| MarkdownFileData {
                _path: path,
                _content: content.clone(),
            })
            .collect::<Vec<_>>()
    }
}

impl ClassifyByNoteClass for MarkdownFileData {
    type Out = MarkdownFileData;

    type Err = String;

    fn classify_by_note_class(&self) -> Result<(Self::Out, NoteClass), Self::Err> {
        todo!()
    }
}
