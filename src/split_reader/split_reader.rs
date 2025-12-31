use anyhow::Result;
use roxmltree::{Document, NodeId};
use std::{fs, str::FromStr};
use strum_macros::EnumString;

#[derive(EnumString, Debug)]
pub enum Split {
    Chapter9Checkpoint1,
    Chapter9Checkpoint2,
    Chapter9Checkpoint3,
    Chapter9Checkpoint4,
    Chapter9Checkpoint5,
    Chapter9Checkpoint6,
    Chapter9Checkpoint7,
    Chapter9Checkpoint8,
    Chapter9,
}

#[derive(Debug)]
pub struct SplitData {
    auto_reset: bool,
    set_high_priority: bool,
    set_game_time: bool,
    file_time_offset: bool,
    splits: Vec<Split>,
}

pub fn read_splits(file_path: &str) -> Result<SplitData> {
    let doc_text = fs::read_to_string(file_path)?;
    let doc = Document::parse(doc_text.as_str())?;
    let mut splits: Vec<Split> = vec![];
    let mut auto_reset = false;
    let mut set_high_priority = false;
    let mut set_game_time = false;
    let mut file_time_offset = false;
    for child in doc
        .get_node(NodeId::new(0))
        .unwrap()
        .first_child()
        .unwrap()
        .children()
    {
        if child.tag_name().name() == "AutoSplitterSettings" {
            for child2 in child.children() {
                match child2.tag_name().name() {
                    "AutoReset" => auto_reset = child2.text() == Some("True"),
                    "SetHighPriority" => set_high_priority = child2.text() == Some("True"),
                    "SetGameTime" => set_game_time = child2.text() == Some("True"),
                    "FileTimeOffset" => file_time_offset = child2.text() == Some("True"),
                    "Splits" => {
                        for split in child2.children() {
                            if split.tag_name().name() == "Split" {
                                if let Some(split_name) = split.text()
                                    && let Ok(split_obj) = Split::from_str(split_name)
                                {
                                    splits.push(split_obj);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(SplitData {
        auto_reset,
        set_high_priority,
        set_game_time,
        file_time_offset,
        splits,
    })
}
