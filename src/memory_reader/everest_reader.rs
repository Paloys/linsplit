use crate::memory_reader::flags::{AutoSplitterChapterFlags, AutoSplitterFileFlags};
use crate::memory_reader::mem_reader::MemReader;
use anyhow::{Result, anyhow};
use procfs::process::{MMPermissions, Process};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    thread::sleep,
    time::Duration,
};

pub(super) struct EverestMemReader {
    memory: File,
    offset: u64,
}

impl EverestMemReader {
    pub fn new() -> Result<Option<Box<Self>>> {
        const CORE_AUTOSPLITTER_MAGIC: &[u8] = b"EVERESTAUTOSPLIT\xF0\xF1\xF2\xF3";
        const CORE_AUTOSPLITTER_INFO_MIN_VERSION: u8 = 3;
        loop {
            let all_processes: Vec<Process> = procfs::process::all_processes()
                .expect("Can't read /proc")
                .filter_map(|p| match p {
                    Ok(p) => {
                        if p.stat().ok()?.comm.contains("Celeste") {
                            return Some(p);
                        }
                        None
                    }
                    Err(_) => None,
                })
                .collect();
            for process in all_processes {
                if let Ok(mut memory) = process.mem()
                    && let Ok(maps) = process.maps()
                {
                    for map in maps {
                        if map.perms.contains(MMPermissions::READ) {
                            memory.seek(SeekFrom::Start(map.address.0))?;
                            let mut buf: [u8; 20] = [0u8; 20];
                            memory.read_exact(&mut buf).unwrap_or(());
                            if buf.iter().eq(CORE_AUTOSPLITTER_MAGIC) {
                                let mut buf2: [u8; 1] = [0];
                                memory.seek(SeekFrom::Current(0x03))?;
                                memory.read_exact(&mut buf2).unwrap_or(());
                                if u8::from_be_bytes(buf2) < CORE_AUTOSPLITTER_INFO_MIN_VERSION {
                                    continue;
                                }
                                return Ok(Some(Box::new(Self {
                                    memory,
                                    offset: map.address.0,
                                })));
                            }
                        }
                    }
                }
            }
            sleep(Duration::from_millis(200));
        }
    }

    fn read_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory.seek(SeekFrom::Start(self.offset + offset))?;
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
}

impl MemReader for EverestMemReader {
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
