use std::{thread::sleep, time::Duration};

use crate::livesplitone::commands::{Command, TimeSpan, TimingMethod};
use crate::livesplitone::livesplitone::SplitterSocket;
use crate::memory_reader::game_data::GameData;
use crate::split_reader::split_reader::SplitData;
use crate::splitter_logic::splitter_logic::Splitter;

pub struct LinSplitData {
    splits: SplitData,
    socket: SplitterSocket,
    game_data: GameData,
}

impl LinSplitData {
    pub async fn new(file_path: &str, addr: &str, save_location: &str) -> Self {
        let splits = SplitData::read_splits(file_path).unwrap();
        let socket = SplitterSocket::new(addr).await.unwrap();
        let game_data = GameData::new(save_location);
        // tokio::time::sleep(Duration::from_secs(3)).await;
        LinSplitData {
            splits,
            socket,
            game_data,
        }
    }

    pub async fn main_loop(&mut self) {
        let elapsed_offset = self.game_data.game_time;
        let mut current_split: i32 = -1;
        let mut last_level_name: String = Default::default();
        let mut level_started: String = Default::default();
        let mut level_timer: f64 = 0.;
        let mut last_chapter_started = false;
        let mut last_elapsed = 0.;
        if self.splits.set_game_time {
            self.socket
                .send_command(Command::SetCurrentTimingMethod {
                    timing_method: TimingMethod::GameTime,
                })
                .await
                .unwrap();
        }
        loop {
            self.game_data.update();
            let mut should_split = false;
            if current_split == -1 && (self.splits.splits.len() == 0 || self.splits.chapter_splits)
            {
                if self.splits.splits.len() == 0 {
                    let level_name = &self.game_data.level_name;

                    should_split =
                        level_name != "" && last_level_name != "" && *level_name != last_level_name;

                    if should_split {
                        level_started = last_level_name.clone();
                        level_timer = self.game_data.level_time;
                    }
                    last_level_name = level_name.clone();
                } else if !self.splits.il_splits {
                    should_split = self.game_data.starting_new_file;
                } else {
                    let chapter_started = self.game_data.chapter_started;

                    should_split = chapter_started && !last_chapter_started;

                    last_chapter_started = chapter_started;
                }
            } else {
                let completed = self.game_data.chapter_complete;
                let area_id = self.game_data.area_id;
                let elapsed: f64 = {
                    if self.splits.file_time_offset {
                        self.game_data.game_time - elapsed_offset
                    } else {
                        if self.splits.il_splits {
                            if self.game_data.area_id == -1 {
                                last_elapsed
                            } else {
                                self.game_data.level_time
                            }
                        } else {
                            self.game_data.game_time
                        }
                    }
                };
                let area_difficulty = self.game_data.area_difficulty;
                let add_amount =
                    (self.splits.splits.len() > 0 && !self.splits.chapter_splits) as i32;
                let opt_split = self
                    .splits
                    .splits
                    .get((current_split + add_amount) as usize);
                let mut level_name = &self.game_data.level_name;
                if level_name == "" && area_id == -1 {
                    level_name = &last_level_name
                };
                // will be useful later
                // let cassettes = self.game_data.cassettes;
                // let heart_gems = self.game_data.heart_gems;
                // let chapter_cassette = self.game_data.chapter_cassette_collected;
                // let chapter_heart = self.game_data.chapter_heart_collected;

                if let Some(split) = opt_split {
                    // TODO: add manual split
                    should_split = split.should_split()(&self.game_data);
                    // TODO: add last cassettes, heartsgems, areaID, areaDifficulty for full support
                }

                if should_split && add_amount > 0 && current_split < 0 {
                    level_timer = self.game_data.level_time;
                }

                // TODO: Add last_completed and level_name
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
            let should_reset =
                self.splits.auto_reset && self.splits.il_splits && self.game_data.area_id == -1;
            if should_reset {
                self.socket
                    .send_command(Command::Reset {
                        save_attempt: Some(true),
                    })
                    .await
                    .unwrap();
                current_split = -1;
            } else if should_split {
                self.socket
                    .send_command(Command::SplitOrStart)
                    .await
                    .unwrap();
                current_split += 1;
            }

            sleep(Duration::from_millis(1));
        }
    }
}
