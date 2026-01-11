use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify, RwLock};

use crate::livesplitone::commands::{Command, Event, TimeSpan, TimingMethod};
use crate::livesplitone::livesplitone::SplitterSocket;
use crate::memory_reader::game_data::GameData;
use crate::split_reader::split_reader::{Area, AreaMode, Split, SplitData};

pub struct LinSplitData {
    splits: SplitData,
    socket: Arc<SplitterSocket>,
    game_data: RwLock<GameData>,
    events: Arc<Mutex<VecDeque<Event>>>,
    event_notifications: Arc<Notify>,
    exiting_chapter: Mutex<bool>,
    current_split: Mutex<i32>,
    last_area_id: Mutex<Area>,
    last_area_difficulty: Mutex<AreaMode>,
}

impl LinSplitData {
    pub async fn new(file_path: &str, addr: &str, save_location: String) -> Arc<Self> {
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let event_notifications = Arc::new(Notify::new());
        let splits = SplitData::read_splits(file_path).unwrap();
        let socket =
            SplitterSocket::new(addr, Arc::clone(&events), Arc::clone(&event_notifications))
                .await
                .unwrap();
        let game_data = RwLock::new(GameData::new(save_location).await);
        // tokio::time::sleep(Duration::from_secs(3)).await;
        let data = Arc::new(LinSplitData {
            splits,
            socket,
            game_data,
            exiting_chapter: Mutex::new(false),
            events,
            event_notifications,
            current_split: Mutex::new(-1),
            last_area_id: Mutex::new(Area::Unknown),
            last_area_difficulty: Mutex::new(AreaMode::Unknown),
        });
        let data_loop = Arc::clone(&data);
        tokio::spawn(async move { data_loop.event_loop().await });
        return data;
    }

    async fn event_loop(self: Arc<Self>) {
        loop {
            self.event_notifications.notified().await;
            if let Some(event) = self.events.lock().await.pop_front() {
                match event {
                    Event::Started => {
                        *self.current_split.lock().await = 0;
                    }
                    Event::Splitted | Event::Finished => {
                        *self.current_split.lock().await += 1;
                        *self.exiting_chapter.lock().await = false;
                    }
                    Event::Reset => {
                        *self.current_split.lock().await = -1;
                        *self.exiting_chapter.lock().await = false;
                        *self.last_area_id.lock().await = Area::Unknown;
                        *self.last_area_difficulty.lock().await = AreaMode::Unknown;
                    }
                    Event::SplitUndone => {
                        *self.current_split.lock().await -= 1;
                        *self.exiting_chapter.lock().await = false;
                    }
                    Event::SplitSkipped => {
                        *self.current_split.lock().await += 1;
                        *self.exiting_chapter.lock().await = false;
                    }
                    _ => {}
                }
            }
        }
    }

    async fn chapter_split(
        &self,
        area_id: Area,
        chapter_area: Area,
        level: &str,
        completed: bool,
        last_completed: bool,
    ) -> bool {
        let mut exiting_chapter = self.exiting_chapter.lock().await;
        if !*exiting_chapter {
            let level_name = if chapter_area == Area::TheSummit {
                Some(level)
            } else {
                None
            };
            *exiting_chapter = area_id == chapter_area
                && completed
                && !last_completed
                && (chapter_area != Area::TheSummit
                    || (level_name
                        .is_some_and(|name| !name.to_lowercase().starts_with("credits"))));
            return *exiting_chapter && self.splits.il_splits;
        }
        !completed && last_completed
    }

    async fn area_complete_split(
        &self,
        area: &str,
        area_id: Area,
        level: &str,
        completed: bool,
        last_completed: bool,
    ) -> bool {
        let split_info: Vec<&str> = area.split("-").collect();
        match split_info[..] {
            [chapter] => {
                if let Ok(split_area) = Area::from_str(chapter.trim()) {
                    return self
                        .chapter_split(area_id, split_area, level, completed, last_completed)
                        .await;
                }
                return false;
            }
            [chapter, difficulty] => {
                if let Ok(split_area) = Area::from_str(chapter.trim())
                    && let Ok(area_difficulty) = AreaMode::from_str(difficulty.trim())
                {
                    return self
                        .chapter_split(area_id, split_area, level, completed, last_completed)
                        .await
                        && area_difficulty == *self.last_area_difficulty.lock().await;
                }
                return false;
            }
            _ => {
                return false;
            }
        }
    }

    async fn area_change_split(
        &self,
        area: &str,
        curr_area_id: Area,
        area_id_to_check: Area,
        curr_area_difficulty: AreaMode,
        area_difficulty_to_check: AreaMode,
    ) -> bool {
        let split_info: Vec<&str> = area.split("-").collect();
        match split_info[..] {
            [chapter] => {
                if let Ok(split_area_id) = Area::from_str(chapter.trim()) {
                    return curr_area_id != *self.last_area_id.lock().await
                        && area_id_to_check == split_area_id;
                }
            }
            [chapter, difficulty] => {
                if let Ok(split_area) = Area::from_str(chapter.trim())
                    && let Ok(area_difficulty) = AreaMode::from_str(difficulty.trim())
                {
                    return curr_area_id != *self.last_area_id.lock().await
                        && area_id_to_check == split_area
                        && curr_area_difficulty != *self.last_area_difficulty.lock().await
                        && area_difficulty_to_check == area_difficulty;
                }
            }
            _ => {}
        };
        false
    }

    #[rustfmt::skip]
    pub async fn main_loop(&self) {
        let elapsed_offset = self.game_data.read().await.game_time;
        let mut last_level_name: String = Default::default();
        let mut _level_started: String = Default::default(); // Might be useful for later, cf https://github.com/ShootMe/LiveSplit.Celeste/blob/5c5bcb2c1456ee04a241575608febb9d35f69084/SplitterComponent.cs#L185
        let mut level_timer: f64 = 0.;
        let mut last_chapter_started = false;
        let mut last_elapsed = 0.;
        let mut last_completed = false;
        let mut last_cassettes = 10000;
        let mut last_heart_gems = 10000;
        let mut last_area_difficulty = AreaMode::Unknown;
        if self.splits.set_game_time {
            self.socket
                .send_command(Command::SetCurrentTimingMethod {
                    timing_method: TimingMethod::GameTime,
                })
                .await
                .unwrap();
        }
        loop {
            self.game_data.write().await.update();
            let mut should_split = false;
            if *self.current_split.lock().await == -1 && (self.splits.splits.len() == 0 || self.splits.chapter_splits)
            {
                if self.splits.splits.len() == 0 {
                    let level_name = self.game_data.read().await.level_name.clone();

                    should_split =
                        level_name != "" && last_level_name != "" && *level_name != last_level_name;

                    if should_split {
                        _level_started = last_level_name.clone();
                        level_timer = self.game_data.read().await.level_time;
                    }
                    last_level_name = level_name.clone();
                } else if !self.splits.il_splits {
                    should_split = self.game_data.read().await.starting_new_file;
                } else {
                    let chapter_started = self.game_data.read().await.chapter_started;

                    should_split = chapter_started && !last_chapter_started;

                    last_chapter_started = chapter_started;
                }
            } else {
                let completed = self.game_data.read().await.chapter_complete;
                let area_id = self.game_data.read().await.area_id;
                let elapsed: f64 = {
                    if self.splits.file_time_offset {
                        self.game_data.read().await.game_time - elapsed_offset
                    } else {
                        if self.splits.il_splits {
                            if self.game_data.read().await.area_id == Area::Menu {
                                last_elapsed
                            } else {
                                self.game_data.read().await.level_time
                            }
                        } else {
                            self.game_data.read().await.game_time
                        }
                    }
                };
                let area_difficulty = self.game_data.read().await.area_difficulty;
                let add_amount =
                    (self.splits.splits.len() > 0 && !self.splits.chapter_splits) as i32;
                let opt_split = self
                    .splits
                    .splits
                    .get((*self.current_split.lock().await + add_amount) as usize);
                let mut level_name = self.game_data.read().await.level_name.clone();
                if level_name == "" && area_id == Area::Menu {
                    level_name = last_level_name.clone()
                };
                let cassettes = self.game_data.read().await.cassettes;
                let heart_gems = self.game_data.read().await.heart_gems;
                let chapter_cassette = self.game_data.read().await.chapter_cassette_collected;
                let chapter_heart = self.game_data.read().await.chapter_heart_collected;
                //println!("{:?}", opt_split);
                if let Some(split) = opt_split {
                    match split {
                        Split::Manual => {}
                        Split::LevelEnter { level } => should_split = area_id != Area::Menu && level_name != last_level_name && level.to_lowercase() == level_name.to_lowercase(),
                        Split::LevelExit { level } => should_split = area_id != Area::Menu && level_name != last_level_name && level.to_lowercase() == last_level_name.to_lowercase(),
                        Split::ChapterA => {
                            should_split = self.chapter_split(
                                Area::Prologue,
                                Area::Prologue,
                                &level_name,
                                completed,
                                last_completed,
                            ).await
                        }
                        Split::AreaComplete { area } => {
                            let area = area.clone(); // TODO: find clean way to do that
                            should_split = self.area_complete_split(
                                &area,
                                area_id,
                                &level_name,
                                completed,
                                last_completed,
                            ).await
                        }
                        Split::AreaOnEnter { area } => {
                            let area = area.clone(); // TODO: same as above
                            should_split = self.area_change_split(
                                &area,
                                area_id,
                                area_id,
                                area_difficulty,
                                area_difficulty,
                            ).await
                        }
                        Split::AreaOnExit { area } => {
                            let area = area.clone(); // TODO: same as above
                            should_split = self.area_change_split(
                                &area,
                                area_id,
                                *self.last_area_id.lock().await,
                                area_difficulty,
                                last_area_difficulty,
                            ).await
                        }
                        Split::Prologue => should_split = self.chapter_split(area_id, Area::Prologue, &level_name, completed, last_completed).await,
                        Split::Chapter1 => should_split = self.chapter_split(area_id, Area::ForsakenCity, &level_name, completed, last_completed).await,
                        Split::Chapter2 => should_split = self.chapter_split(area_id, Area::OldSite, &level_name, completed, last_completed).await,
                        Split::Chapter3 => should_split = self.chapter_split(area_id, Area::CelestialResort, &level_name, completed, last_completed).await,
                        Split::Chapter4 => should_split = self.chapter_split(area_id, Area::GoldenRidge, &level_name, completed, last_completed).await,
                        Split::Chapter5 => should_split = self.chapter_split(area_id, Area::MirrorTemple, &level_name, completed, last_completed).await,
                        Split::Chapter6 => should_split = self.chapter_split(area_id, Area::Reflection, &level_name, completed, last_completed).await,
                        Split::Chapter7 => should_split = self.chapter_split(area_id, Area::TheSummit, &level_name, completed, last_completed).await,
                        Split::Epilogue => should_split = self.chapter_split(area_id, Area::Epilogue, &level_name, completed, last_completed).await,
                        Split::Chapter8 => should_split = self.chapter_split(area_id, Area::Core, &level_name, completed, last_completed).await,
                        Split::Chapter9 => should_split = self.chapter_split(area_id, Area::Farewell, &level_name, completed, last_completed).await,
                        Split::Chapter1Checkpoint1 => should_split = area_id == Area::ForsakenCity && ((area_difficulty == AreaMode::ASide && level_name == "6") || (area_difficulty != AreaMode::ASide && level_name == "04")),
                        Split::Chapter1Checkpoint2 => should_split = area_id == Area::ForsakenCity && ((area_difficulty == AreaMode::ASide && level_name == "9b") || (area_difficulty != AreaMode::ASide && level_name == "08")),
                        Split::Chapter2Checkpoint1 => should_split = area_id == Area::OldSite && ((area_difficulty == AreaMode::ASide && level_name == "3") || (area_difficulty != AreaMode::ASide && level_name == "03")),
                        Split::Chapter2Checkpoint2 => should_split = area_id == Area::OldSite && ((area_difficulty == AreaMode::ASide && level_name == "end_3") || (area_difficulty != AreaMode::ASide && level_name == "08b")),
                        Split::Chapter3Checkpoint1 => should_split = area_id == Area::CelestialResort && ((area_difficulty == AreaMode::ASide && level_name == "08-a") || (area_difficulty != AreaMode::ASide && level_name == "06")),
                        Split::Chapter3Checkpoint2 => should_split = area_id == Area::CelestialResort && ((area_difficulty == AreaMode::ASide && level_name == "09-d") || (area_difficulty != AreaMode::ASide && level_name == "11")),
                        Split::Chapter3Checkpoint3 => should_split = area_id == Area::CelestialResort && ((area_difficulty == AreaMode::ASide && level_name == "00-d") || (area_difficulty != AreaMode::ASide && level_name == "16")),
                        Split::Chapter4Checkpoint1 => should_split = area_id == Area::GoldenRidge && level_name == "b-00",
                        Split::Chapter4Checkpoint2 => should_split = area_id == Area::GoldenRidge && level_name == "c-00",
                        Split::Chapter4Checkpoint3 => should_split = area_id == Area::GoldenRidge && level_name == "d-00",
                        Split::Chapter5Checkpoint1 => should_split = area_id == Area::MirrorTemple && level_name == "b-00",
                        Split::Chapter5Checkpoint2 => should_split = area_id == Area::MirrorTemple && level_name == "c-00",
                        Split::Chapter5Checkpoint3 => should_split = area_id == Area::MirrorTemple && level_name == "d-00",
                        Split::Chapter5Checkpoint4 => should_split = area_id == Area::MirrorTemple && level_name == "e-00",
                        Split::Chapter6Checkpoint1 => should_split = area_id == Area::Reflection && ((area_difficulty == AreaMode::ASide && level_name == "00") || (area_difficulty != AreaMode::ASide && level_name == "b-00")),
                        Split::Chapter6Checkpoint2 => should_split = area_id == Area::Reflection && ((area_difficulty == AreaMode::ASide && level_name == "04") || (area_difficulty != AreaMode::ASide && level_name == "c-00")),
                        Split::Chapter6Checkpoint3 => should_split = area_id == Area::Reflection && ((area_difficulty == AreaMode::ASide && level_name == "b-00") || (area_difficulty != AreaMode::ASide && level_name == "d-00")),
                        Split::Chapter6Checkpoint4 => should_split = area_id == Area::Reflection && level_name == "boss-00",
                        Split::Chapter6Checkpoint5 => should_split = area_id == Area::Reflection && level_name == "after-00",
                        Split::Chapter7Checkpoint1 => should_split = area_id == Area::TheSummit && level_name == "b-00",
                        Split::Chapter7Checkpoint2 => should_split = area_id == Area::TheSummit && ((area_difficulty == AreaMode::ASide && level_name == "c-00") || (area_difficulty != AreaMode::ASide && level_name == "c-01")),
                        Split::Chapter7Checkpoint3 => should_split = area_id == Area::TheSummit && level_name == "d-00",
                        Split::Chapter7Checkpoint4 => should_split = area_id == Area::TheSummit && ((area_difficulty == AreaMode::ASide && level_name == "e-00b") || (area_difficulty != AreaMode::ASide && level_name == "e-00")),
                        Split::Chapter7Checkpoint5 => should_split = area_id == Area::TheSummit && level_name == "f-00",
                        Split::Chapter7Checkpoint6 => should_split = area_id == Area::TheSummit && level_name == "g-00",
                        Split::Chapter8Checkpoint1 => should_split = area_id == Area::Core && level_name == "a-00",
                        Split::Chapter8Checkpoint2 => should_split = area_id == Area::Core && ((area_difficulty == AreaMode::ASide && level_name == "c-00") || (area_difficulty != AreaMode::ASide && level_name == "b-00")),
                        Split::Chapter8Checkpoint3 => should_split = area_id == Area::Core && ((area_difficulty == AreaMode::ASide && level_name == "d-00") || (area_difficulty != AreaMode::ASide && level_name == "c-01")),
                        Split::Chapter9Checkpoint1 => should_split = area_id == Area::Farewell && level_name == "a-00",
                        Split::Chapter9Checkpoint2 => should_split = area_id == Area::Farewell && level_name == "c-00",
                        Split::Chapter9Checkpoint3 => should_split = area_id == Area::Farewell && level_name == "e-00z",
                        Split::Chapter9Checkpoint4 => should_split = area_id == Area::Farewell && level_name == "f-door",
                        Split::Chapter9Checkpoint5 => should_split = area_id == Area::Farewell && level_name == "h-00b",
                        Split::Chapter9Checkpoint6 => should_split = area_id == Area::Farewell && level_name == "i-00",
                        Split::Chapter9Checkpoint7 => should_split = area_id == Area::Farewell && level_name == "j-00",
                        Split::Chapter9Checkpoint8 => should_split = area_id == Area::Farewell && level_name == "j-16",
                        Split::HeartGemAny => should_split = ((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1,
                        Split::Chapter1Cassette => should_split = area_id == Area::ForsakenCity && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter1HeartGem => should_split = area_id == Area::ForsakenCity && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter2Cassette => should_split = area_id == Area::OldSite && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter2HeartGem => should_split = area_id == Area::OldSite && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter3Cassette => should_split = area_id == Area::CelestialResort && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter3HeartGem => should_split = area_id == Area::CelestialResort && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter4Cassette => should_split = area_id == Area::GoldenRidge && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter4HeartGem => should_split = area_id == Area::GoldenRidge && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter5Cassette => should_split = area_id == Area::MirrorTemple && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter5HeartGem => should_split = area_id == Area::MirrorTemple && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter6Cassette => should_split = area_id == Area::Reflection && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter6HeartGem => should_split = area_id == Area::Reflection && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter7Cassette => should_split = area_id == Area::TheSummit && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter7HeartGem => should_split = area_id == Area::TheSummit && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                        Split::Chapter8Cassette => should_split = area_id == Area::Core && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_cassette) || cassettes == last_cassettes + 1),
                        Split::Chapter8HeartGem => should_split = area_id == Area::Core && (((self.splits.il_splits || self.splits.file_time_offset) && chapter_heart) || heart_gems == last_heart_gems + 1),
                    }
                    last_cassettes = cassettes;
                    last_heart_gems = heart_gems;
                    *self.last_area_id.lock().await = area_id;
                    last_area_difficulty = area_difficulty;
                }

                if should_split && add_amount > 0 && *self.current_split.lock().await < 0 {
                    level_timer = self.game_data.read().await.level_time;
                }

                last_completed = completed;
                last_level_name = level_name;

                if elapsed > 0. || last_elapsed == elapsed {
                    self.socket
                        .send_command(Command::SetGameTime {
                            time: TimeSpan::from_seconds(
                                if self.splits.splits.len() == 0 || add_amount > 0 {
                                    elapsed - level_timer
                                } else {
                                    elapsed
                                },
                            ),
                        })
                        .await
                        .unwrap();
                }

                last_elapsed = elapsed;
            }
            let should_reset = self.splits.auto_reset
                && self.splits.il_splits
                && self.game_data.read().await.area_id == Area::Menu;
            let mut chap = self.exiting_chapter.lock().await;
            if should_reset {
                self.socket
                    .send_command(Command::Reset {
                        save_attempt: Some(true),
                    })
                    .await
                    .unwrap();
                *chap = false;
            } else if should_split {
                self.socket
                    .send_command(Command::SplitOrStart)
                    .await
                    .unwrap();
                *chap = false;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
}
