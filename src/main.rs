use std::{thread::sleep, time::Duration};
mod memory_reader;
use crate::memory_reader::reader::{GameData};

#[cfg(target_os = "linux")]
fn main() {
    let mut gamedata = GameData::new();
    loop {
        gamedata.update();
        println!("{}", gamedata);
        sleep(Duration::from_millis(1));
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
