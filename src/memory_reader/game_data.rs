use std::fmt::{self, Display};
use std::fs::read;

use crate::memory_reader::mem_reader;

use super::everest_reader::EverestMemReader;
use super::mem_reader::MemReader;
use super::vanilla_reader::VanillaMemReader;

pub struct GameData {
    mem_reader: Box<dyn MemReader>,
    pub chapter_complete: bool,
    pub level_name: String,
    pub area_id: i32,
    pub area_difficulty: i32,
    pub chapter_started: bool,
    pub game_time: f64,
    pub level_time: f64,
    pub strawberries: u32,
    pub cassettes: u32,
    pub chapter_cassette_collected: bool,
    pub heart_gems: u32,
    pub chapter_heart_collected: bool,
    pub starting_new_file: bool,
}

impl GameData {
    pub fn new(save_location: &str) -> Self {
        println!("Waiting for Celeste...");
        let mem_reader: Box<dyn MemReader>;
        loop {
            if let Ok(Some(reader)) = VanillaMemReader::new(save_location) {
                println!("Found Vanilla Celeste.");
                mem_reader = reader;
                break;
            } else if let Ok(Some(reader)) = EverestMemReader::new() {
                println!("Found Everest.");
                mem_reader = reader;
                break;
            }
        }
        Self {
            mem_reader,
            chapter_complete: false,
            level_name: String::new(),
            area_id: 0,
            area_difficulty: 0,
            chapter_started: false,
            game_time: 0.0,
            level_time: 0.0,
            strawberries: 0,
            cassettes: 0,
            chapter_cassette_collected: false,
            heart_gems: 0,
            chapter_heart_collected: false,
            starting_new_file: false,
        }
    }

    pub fn update(&mut self) {
        self.chapter_complete = self.mem_reader.chapter_complete().unwrap_or(false);
        self.level_name = self
            .mem_reader
            .level_name()
            .unwrap_or(String::from("Unknown"));
        self.area_id = self.mem_reader.area_id().unwrap_or(-2);
        self.area_difficulty = self.mem_reader.area_difficulty().unwrap_or(0);
        self.chapter_started = self.mem_reader.chapter_started().unwrap_or(false);
        self.game_time = self.mem_reader.game_time().unwrap_or(0.0);
        self.level_time = self.mem_reader.level_time().unwrap_or(0.0);
        self.strawberries = self.mem_reader.strawberries().unwrap_or(0);
        self.cassettes = self.mem_reader.cassettes().unwrap_or(0);
        self.chapter_cassette_collected = self
            .mem_reader
            .chapter_cassette_collected()
            .unwrap_or(false);
        self.heart_gems = self.mem_reader.heart_gems().unwrap_or(0);
        self.chapter_heart_collected = self.mem_reader.chapter_heart_collected().unwrap_or(false);
        // self.starting_new_file = self.mem_reader.starting_new_file().unwrap_or(false);
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            format!("chapter_complete: {}", self.chapter_complete),
            format!("level_name: {}", self.level_name),
            format!("area_id: {}", self.area_id),
            format!("area_difficulty: {}", self.area_difficulty),
            format!("chapter_started: {}", self.chapter_started),
            format!("game_time: {:.3}", self.game_time),
            format!("level_time: {:.3}", self.level_time),
            format!("strawberries: {}", self.strawberries),
            format!("cassettes: {}", self.cassettes),
            format!(
                "chapter_cassette_collected: {}",
                self.chapter_cassette_collected
            ),
            format!("heart_gems: {}", self.heart_gems),
            format!("chapter_heart_collected: {}", self.chapter_heart_collected),
            //format!("starting_new_file: {}", self.starting_new_file),
        )
    }
}
