use crate::memory_reader::reader::GameData;
use crate::split_reader::split_reader::Split;

pub trait Splitter: Sized {
    fn should_split(&self) -> impl Fn(&GameData) -> bool;
}

#[rustfmt::skip]
impl Splitter for Split {
    fn should_split(&self) -> impl Fn(&GameData) -> bool {
        match self {
            Split::Chapter9            => |game_data: &GameData| {game_data.area_id == 10 && game_data.chapter_complete}, 
            Split::Chapter9Checkpoint1 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "a-00"},
            Split::Chapter9Checkpoint2 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "c-00"},
            Split::Chapter9Checkpoint3 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "e-00z"},
            Split::Chapter9Checkpoint4 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "f-door"},
            Split::Chapter9Checkpoint5 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "h-00b"},
            Split::Chapter9Checkpoint6 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "i-00"},
            Split::Chapter9Checkpoint7 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "j-00"},
            Split::Chapter9Checkpoint8 => |game_data: &GameData| {game_data.area_id == 10 && game_data.level_name == "j-16"},
        }
    }
}
