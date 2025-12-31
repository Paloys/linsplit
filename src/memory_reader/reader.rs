use crate::memory_reader::flags::{AutoSplitterChapterFlags, AutoSplitterFileFlags};
use anyhow::{Result, anyhow};
use procfs::process::{MMPermissions, MemoryMap, Process};
use std::{
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{Read, Seek, SeekFrom}, thread::{self, sleep}, time::Duration,
};

pub struct GameData {
    mem_reader: EverestMemReader,
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
    pub fn new() -> Self {
        Self {
            mem_reader: EverestMemReader::new().expect("Error getting the process"),
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
        self.starting_new_file = self.mem_reader.starting_new_file().unwrap_or(false);
    }
}

impl Display for GameData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
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
            format!("starting_new_file: {}", self.starting_new_file),
        )
    }
}

struct EverestMemReader {
    _process: Process,
    map: MemoryMap,
    memory: File,
    _hooked: bool,
}

#[derive(Debug, Clone)]
pub struct NotFoundError;

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "process to attach not found")
    }
}

impl Error for NotFoundError {}

impl EverestMemReader {
    fn new() -> Result<Self> {
        const CORE_AUTOSPLITTER_MAGIC: &[u8] = b"EVERESTAUTOSPLIT\xF0\xF1\xF2\xF3";
        const CORE_AUTOSPLITTER_INFO_MIN_VERSION: u8 = 3;
        println!("Waiting for Celeste...");
        loop {
            let all_processes: Vec<Process> = procfs::process::all_processes()
                .expect("Can't read /proc")
                .filter_map(|p| match p {
                    Ok(p) => {
                        if p.stat().ok()?.comm.contains("Celeste") {
                            return Some(p);
                        }
                        None
                    } // happy path
                    Err(_) => None,
                })
                .collect();
            for process in all_processes {
                if let Ok(mut memory) = process.mem()
                    && let Ok(unwarped_maps) = process.maps()
                {
                    for map in unwarped_maps {
                        if map.perms.contains(MMPermissions::READ) {
                            memory.seek(SeekFrom::Start(map.address.0))?;
                            let mut buf: [u8; 20] = [0u8; 20];
                            memory.read_exact(&mut buf).unwrap_or(());
                            if buf.iter().eq(CORE_AUTOSPLITTER_MAGIC) {
                                let mut buf2: [u8; 1] = [0];
                                memory.seek(SeekFrom::Current(0x03))?;
                                memory.read_exact(&mut buf2).unwrap_or(());
                                if u8::from_be_bytes(buf2) < CORE_AUTOSPLITTER_INFO_MIN_VERSION {
                                    // println!("Bruh : {:?}", buf2);
                                    continue;
                                }
                                return Ok(Self {
                                    _process: process,
                                    map,
                                    memory,
                                    _hooked: true,
                                });
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_millis(200));
        }
    }

    fn read_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory
            .seek(SeekFrom::Start(self.map.address.0 + offset))?;
        let mut buf = [0; COUNT];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_vec_global_bits(&mut self, offset: u64, count: usize) -> Result<Vec<u8>> {
        self.memory.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0; count];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_global_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory.seek(SeekFrom::Start(offset))?;
        let mut buf = [0; COUNT];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn chapter_complete(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("a"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_COMPLETE),
        )
    }

    fn level_name(&mut self) -> Result<String> {
        let level_name_str = u64::from_le_bytes(self.read_bits(0x38)?);
        if level_name_str < 2 {
            return Err(anyhow!("failed to get level_name_str"));
        }
        let name_len = u16::from_le_bytes(self.read_global_bits(level_name_str - 2)?);
        Ok(String::from_utf8(self.read_vec_global_bits(
            level_name_str,
            name_len as usize,
        )?)?)
    }

    fn area_id(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bits(0x30)?))
    }

    fn area_difficulty(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bits(0x34)?))
    }

    fn chapter_started(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("failed"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_STARTED),
        )
    }

    fn game_time(&mut self) -> Result<f64> {
        Ok(i64::from_le_bytes(self.read_bits(0x50)?) as f64 / 10000000.)
    }

    fn level_time(&mut self) -> Result<f64> {
        Ok(i64::from_le_bytes(self.read_bits(0x40)?) as f64 / 10000000.)
    }

    fn strawberries(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bits(0x58)?))
    }

    fn cassettes(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bits(0x60)?))
    }

    fn chapter_cassette_collected(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("failed"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_CASSETTE),
        )
    }

    fn heart_gems(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bits(0x64)?))
    }

    fn chapter_heart_collected(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("failed"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_HEART),
        )
    }

    fn starting_new_file(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterFileFlags::from_bits(u32::from_le_bytes(self.read_bits(0x68)?))
                .ok_or(anyhow!("failed"))?
                .contains(AutoSplitterFileFlags::STARTING_NEW_FILE),
        )
    }
}
