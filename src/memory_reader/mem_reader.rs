use anyhow::Result;

use crate::split_reader::split_reader::{Area, AreaMode};

pub trait MemReader {
    fn chapter_complete(&mut self) -> Result<bool>;
    fn level_name(&mut self) -> Result<String>;
    fn area_id(&mut self) -> Result<Area>;
    fn area_difficulty(&mut self) -> Result<AreaMode>;
    fn chapter_started(&mut self) -> Result<bool>;
    fn game_time(&mut self) -> Result<f64>;
    fn level_time(&mut self) -> Result<f64>;
    fn strawberries(&mut self) -> Result<u32>;
    fn cassettes(&mut self) -> Result<u32>;
    fn chapter_cassette_collected(&mut self) -> Result<bool>;
    fn heart_gems(&mut self) -> Result<u32>;
    fn chapter_heart_collected(&mut self) -> Result<bool>;
    fn starting_new_file(&mut self) -> Result<bool>;
}
