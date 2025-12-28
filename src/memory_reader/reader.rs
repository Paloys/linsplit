use crate::memory_reader::flags::AutoSplitterChapterFlags;
use anyhow::{Result, anyhow};
use procfs::process::{MMPermissions, MemoryMap, Process};
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub struct EverestMemReader {
    process: Process,
    map: MemoryMap,
    memory: File,
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
    pub fn new() -> Result<Self> {
        const CORE_AUTOSPLITTER_MAGIC: &[u8] = b"EVERESTAUTOSPLIT\xF0\xF1\xF2\xF3";
        const CORE_AUTOSPLITTER_INFO_MIN_VERSION: u8 = 3;
        let all_processes: Vec<Process> = procfs::process::all_processes()
            .expect("Can't read /proc")
            .filter_map(|p| match p {
                Ok(p) => {
                    if p.stat().ok()?.comm.contains("Celeste") {
                        // TODO: CHECK THE ABOVE IS RIGHT
                        println!(
                            "Scanning process {} (PID {})",
                            p.stat().unwrap().comm,
                            p.stat().unwrap().pid
                        );
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
                                process,
                                map,
                                memory,
                            });
                        }
                    }
                }
            }
        }
        Err(NotFoundError.into())
    }

    fn read_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory
            .seek(SeekFrom::Start(self.map.address.0 + offset))?;
        let mut buf = [0; COUNT];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_vec_bits(&mut self, offset: u64, count: usize) -> Result<Vec<u8>> {
        self.memory
            .seek(SeekFrom::Start(self.map.address.0 + offset))?;
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

    pub fn chapter_complete(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("a"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_COMPLETE),
        )
    }

    pub fn level_name(&mut self) -> Result<String> {
        let level_name_str = u64::from_le_bytes(self.read_bits(0x38)?);
        if level_name_str < 2 {
            return Err(anyhow!("failed to get level_name_str"));
        }
        let name_len = u16::from_le_bytes(self.read_global_bits(level_name_str - 2)?);
        Ok(String::from_utf8(self.read_vec_bits(level_name_str, name_len as usize)?)?)
    }

    pub fn area_id(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bits(0x30)?))
    }

    pub fn area_difficulty(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_bits(0x34)?))
    }

    pub fn chapter_started(&mut self) -> Result<bool> {
        Ok(
            AutoSplitterChapterFlags::from_bits(u32::from_le_bytes(self.read_bits(0x4c)?))
                .ok_or(anyhow!("failed"))?
                .contains(AutoSplitterChapterFlags::CHAPTER_STARTED),
        )
    }

    pub fn game_time(&mut self) -> Result<f64> {
        Ok(i64::from_le_bytes(self.read_bits(0x50)?) as f64 / 10000000.)
    }

    pub fn level_time(&mut self) -> Result<f64> {
        Ok(i64::from_le_bytes(self.read_bits(0x40)?) as f64 / 10000000.)
    }

    pub fn strawberries(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_bits(0x58)?))
    }
}
