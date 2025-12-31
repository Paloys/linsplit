use roxmltree::{Document, NodeId};
use std::io::Seek;
use std::{fs, thread::sleep, time::Duration};
use tokio::io::split;
mod livesplitone;
mod memory_reader;
mod split_reader;
mod splitter_logic;
use crate::livesplitone::commands::{Command, TimeSpan};
use crate::livesplitone::livesplitone::SplitterSocket;
use crate::memory_reader::reader::GameData;
use crate::split_reader::split_reader::SplitData;
use crate::splitter_logic::splitter_logic::Splitter;

struct LinSplitData {
    splits: SplitData,
    socket: SplitterSocket,
    game_data: GameData,
}

impl LinSplitData {
    async fn new(file_path: &str, addr: &str) -> Self {
        let mut splits = SplitData::read_splits(file_path).unwrap();
        let socket = SplitterSocket::new(addr).await.unwrap();
        let game_data = GameData::new();
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("Found Celeste.");
        LinSplitData {
            splits,
            socket,
            game_data,
        }
    }

    async fn main_loop(&mut self) {
        let mut last_chapter_started = false;
        loop {
            self.game_data.update();
            if self.game_data.chapter_started && !last_chapter_started {
                self.socket.send_command(Command::Start).await.unwrap();
                break;
            }
            last_chapter_started = self.game_data.chapter_started;
            sleep(Duration::from_millis(1));
        }
        sleep(Duration::from_secs(2));
        self.game_data.update();
        self.socket.send_command(Command::SetGameTime {time: TimeSpan::from_seconds(self.game_data.level_time)}).await.unwrap();
        let mut splits = self.splits.splits.iter();
        let mut split = splits.next();
        loop {
            self.game_data.update();
            match split {
                Some(split_value) => {
                    if split_value.should_split()(&self.game_data) {
                        self.socket.send_command(Command::Split).await.unwrap();
                        split = splits.next();
                    };
                }
                None => {
                    break;
                }
            };
            sleep(Duration::from_millis(1));
        }
    }
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() {
    // let mut gamedata = GameData::new();
    // loop {
    //     gamedata.update();
    //     println!("{}", gamedata);
    //     sleep(Duration::from_millis(1));
    // }
    let mut data = LinSplitData::new("./Farewell checkpoints.lss", "127.0.0.1:51000").await;
    data.main_loop().await;
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
