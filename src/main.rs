use std::io::Write;
use std::{io, thread::sleep, time::Duration};
mod memory_reader;
use crate::memory_reader::reader::EverestMemReader;

#[cfg(target_os = "linux")]
fn main() {

    let mut reader = EverestMemReader::new().expect("Error getting the process");
    loop {
        print!("\r\x1b[2KGame Time: {:?}; LevelTime: {:?}", reader.game_time(), reader.level_time());
        sleep(Duration::from_millis(1));
        io::stdout().flush().unwrap();
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
