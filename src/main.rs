use roxmltree::{Document, NodeId};
use std::{fs, thread::sleep, time::Duration};
mod memory_reader;
mod split_reader;
mod splitter_logic;
mod livesplitone;
use crate::memory_reader::reader::GameData;
use crate::livesplitone::commands::Command;


#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() {
    // let mut gamedata = GameData::new();
    // loop {
    //     gamedata.update();
    //     println!("{}", gamedata);
    //     sleep(Duration::from_millis(1));
    // }
    let mut socket = livesplitone::livesplitone::SplitterSocket::new("0.0.0.0:51000").await.unwrap();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    println!("{:?}", socket.send_command(Command::Start).await);
    tokio::time::sleep(Duration::from_millis(500)).await;
    println!("{:?}", socket.send_command(Command::Pause).await);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
