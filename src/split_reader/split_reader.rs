use anyhow::Result;
use roxmltree::{Document, NodeId};
use std::{fs, ops::Index, str::FromStr};
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
    pub auto_reset: bool,
    pub set_game_time: bool,
    pub file_time_offset: bool,
    pub il_splits: bool,
    pub chapter_splits: bool,
    pub splits: Vec<Split>,
}

impl SplitData {
    pub fn read_splits(file_path: &str) -> Result<Self> {
        let doc_text = fs::read_to_string(file_path)?;
        let doc = Document::parse(doc_text.as_str())?;
        let mut splits: Vec<Split> = vec![];
        let mut auto_reset = false;
        let mut set_game_time = false;
        let mut file_time_offset = false;
        let mut chapter_count = 0;
        let mut heart_count = 0;
        let mut area_count = 0;
        let mut cassette_count = 0;
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
                        "SetGameTime" => set_game_time = child2.text() == Some("True"),
                        "FileTimeOffset" => file_time_offset = child2.text() == Some("True"),
                        "Splits" => {
                            for split in child2.children() {
                                if split.tag_name().name() == "Split" {
                                    if let Some(split_name) = split.text()
                                        && let Ok(split_obj) = Split::from_str(split_name)
                                    {
                                        splits.push(split_obj);
                                        if split_name.len() == 8 {
                                            chapter_count += 1;
                                        } else if split_name.contains("HeartGem") {
                                            heart_count += 1;
                                        } else if split_name.contains("AreaComplete") {
                                            area_count += 1;
                                        } else if split_name.contains("Cassette") {
                                            cassette_count += 1;
                                        }
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
            set_game_time,
            file_time_offset,
            il_splits: splits.len() == 0
                || (chapter_count <= 1
                    && heart_count <= 1
                    && area_count <= 1
                    && cassette_count <= 1),
            chapter_splits: chapter_count > 0
                || heart_count > 0
                || area_count > 0
                || cassette_count > 0,
            splits,
        })
    }
}
