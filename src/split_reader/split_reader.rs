use anyhow::Result;
use roxmltree::{Document, NodeId};
use std::{fs, str::FromStr};
use strum_macros::EnumString;

#[derive(EnumString, Debug)]
pub enum Split {
    Manual,
    ChapterA,
    AreaComplete,
    AreaOnEnter,
    AreaOnExit,
    HeartGemAny,
    LevelEnter,
    LevelExit,
    Prologue,
    Chapter1Checkpoint1,
    Chapter1Checkpoint2,
    Chapter1,
    Chapter2Checkpoint1,
    Chapter2Checkpoint2,
    Chapter2,
    Chapter3Checkpoint1,
    Chapter3Checkpoint2,
    Chapter3Checkpoint3,
    Chapter3,
    Chapter4Checkpoint1,
    Chapter4Checkpoint2,
    Chapter4Checkpoint3,
    Chapter4,
    Chapter5Checkpoint1,
    Chapter5Checkpoint2,
    Chapter5Checkpoint3,
    Chapter5Checkpoint4,
    Chapter5,
    Chapter6Checkpoint1,
    Chapter6Checkpoint2,
    Chapter6Checkpoint3,
    Chapter6Checkpoint4,
    Chapter6Checkpoint5,
    Chapter6,
    Chapter7Checkpoint1,
    Chapter7Checkpoint2,
    Chapter7Checkpoint3,
    Chapter7Checkpoint4,
    Chapter7Checkpoint5,
    Chapter7Checkpoint6,
    Chapter7,
    Epilogue,
    Chapter8Checkpoint1,
    Chapter8Checkpoint2,
    Chapter8Checkpoint3,
    Chapter8,
    Chapter9Checkpoint1,
    Chapter9Checkpoint2,
    Chapter9Checkpoint3,
    Chapter9Checkpoint4,
    Chapter9Checkpoint5,
    Chapter9Checkpoint6,
    Chapter9Checkpoint7,
    Chapter9Checkpoint8,
    Chapter9,
    Chapter1Cassette,
    Chapter1HeartGem,
    Chapter2Cassette,
    Chapter2HeartGem,
    Chapter3Cassette,
    Chapter3HeartGem,
    Chapter4Cassette,
    Chapter4HeartGem,
    Chapter5Cassette,
    Chapter5HeartGem,
    Chapter6Cassette,
    Chapter6HeartGem,
    Chapter7Cassette,
    Chapter7HeartGem,
    Chapter8Cassette,
    Chapter8HeartGem,
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
