use anyhow::Result;
use roxmltree::{Document, NodeId};
use std::{fs, str::FromStr};
use strum_macros::EnumString;

#[derive(EnumString, Debug, Clone)]
pub enum Split {
    Manual,
    ChapterA,
    AreaComplete { area: String },
    AreaOnEnter { area: String },
    AreaOnExit { area: String },
    HeartGemAny,
    LevelEnter { level: String },
    LevelExit { level: String },
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

impl Split {
    fn from_str_field(split: &str) -> Result<Self> {
        if let Ok(split_obj) = Split::from_str(split) {
            return Ok(split_obj)
        }
        let sep: Vec<&str> = split.split(",").collect();
        if sep.len() != 2 {
            return Err(anyhow::anyhow!("wrong split"));
        }
        if let Some(&split) = sep.get(0)
            && let Ok(split_obj) = Split::from_str(split)
        {
            let area = String::from(sep[1]);
            match split_obj {
                Split::AreaComplete { area: _ } => Ok(Split::AreaComplete { area }),
                Split::AreaOnEnter { area: _ } => Ok(Split::AreaOnEnter { area }),
                Split::AreaOnExit { area: _ } => Ok(Split::AreaOnExit { area }),
                Split::LevelEnter { level: _  } => Ok(Split::LevelEnter { level: area }),
                Split::LevelExit { level: _ } => Ok(Split::LevelExit { level: area }),
                _ => {Err(anyhow::anyhow!("wrong split type"))}
            }
        }
        else {
            Err(anyhow::anyhow!("wrong split type"))
        }
    }
}

#[repr(i32)]
#[derive(PartialEq, Clone, Copy, EnumString, Debug)]
pub enum Area {
    Unknown = -2,
    Menu = -1,
    Prologue = 0,
    ForsakenCity = 1,
    OldSite = 2,
    CelestialResort = 3,
    GoldenRidge = 4,
    MirrorTemple = 5,
    Reflection = 6,
    TheSummit = 7,
    Epilogue = 8,
    Core = 9,
    Farewell = 10,
}

#[repr(i32)]
#[derive(PartialEq, Clone, Copy, EnumString)]
pub enum AreaMode {
    Unknown = -2,
    None = -1,
    ASide = 0,
    BSide = 1,
    CSide = 2,
}

impl TryFrom<i32> for Area {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(Area::Menu),
            0 => Ok(Area::Prologue),
            1 => Ok(Area::ForsakenCity),
            2 => Ok(Area::OldSite),
            3 => Ok(Area::CelestialResort),
            4 => Ok(Area::GoldenRidge),
            5 => Ok(Area::MirrorTemple),
            6 => Ok(Area::Reflection),
            7 => Ok(Area::TheSummit),
            8 => Ok(Area::Epilogue),
            9 => Ok(Area::Core),
            10 => Ok(Area::Farewell),
            _ => Ok(Area::Unknown),
        }
    }
}

impl TryFrom<i32> for AreaMode {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(AreaMode::None),
            0 => Ok(AreaMode::ASide),
            1 => Ok(AreaMode::BSide),
            2 => Ok(AreaMode::CSide),
            _ => Ok(AreaMode::Unknown),
        }
    }
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
                                        && let Ok(split_obj) = Split::from_str_field(split_name)
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
