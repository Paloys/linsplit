use crate::memory_reader::mem_reader::MemReader;
use anyhow::Result;
use expand_tilde::expand_tilde;
use procfs::process::{MMPermissions, MMapPath, Process};
use roxmltree::{Document, NodeId};
use std::fs;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub(super) struct VanillaMemReader {
    memory: File,
    offset: u64,
}

impl VanillaMemReader {
    pub fn new(save_location: &str) -> Result<Option<Box<Self>>> {
        let mut times: Vec<[u8; 8]> = Vec::with_capacity(3);
        for i in 0..3 {
            let file_path = expand_tilde(save_location)?.join(format!("{}.celeste", i));
            if file_path.exists() {
                let t = fs::read_to_string(file_path)?;
                let u = Document::parse(t.as_str());
                if let Ok(doc) = u {
                    for child in doc.get_node(NodeId::new(1)).unwrap().children() {
                        if child.tag_name().name() == "Time"
                            && let Some(time_str) = child.text()
                            && let Ok(time) = time_str.parse::<u64>()
                        {
                            times.push(time.to_le_bytes());
                            break;
                        }
                    }
                }
            }
        }
        if times.is_empty() {
            return Ok(None);
        }
        let time = times[1];
        let all_processes: Vec<Process> = procfs::process::all_processes()
            .expect("Can't read /proc")
            .filter_map(|p| match p {
                Ok(p) => {
                    if p.stat().ok()?.comm.contains("Celeste.bin.x86") {
                        return Some(p);
                    }
                    None
                } //  happy path
                Err(_) => None,
            })
            .collect();
        for process in all_processes {
            if let Ok(mut memory) = process.mem()
                && let Ok(maps) = process.maps()
            {
                for map in maps {
                    if map.perms.contains(
                        MMPermissions::READ | MMPermissions::WRITE | MMPermissions::PRIVATE,
                    ) && map.address.1 - map.address.0 >= 24
                    {
                        if !matches!(map.pathname, MMapPath::Anonymous) {
                            continue;
                        }
                        let size = (map.address.1 - map.address.0) as usize;
                        let mut buf = vec![0u8; size];

                        memory.seek(SeekFrom::Start(map.address.0))?;
                        if let Err(_) = memory.read_exact(&mut buf) {
                            continue;
                        };
                        let needle: [u8; 8] = time;

                        for i in (0..=buf.len() - 24).step_by(8) {
                            if &buf[i..i + 8] == needle && buf[i - 16..i].iter().all(|&b| b == 0) {
                                let position = map.address.0 + i as u64;
                                return Ok(Some(Box::new(VanillaMemReader {
                                    memory,
                                    offset: position - 0x28,
                                })));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn read_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory.seek(SeekFrom::Start(self.offset + offset))?;
        let mut buf = [0; COUNT];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_vec_global_bits(&mut self, offset: u64, count: usize) -> Result<Vec<u16>> {
        self.memory.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0; count];
        self.memory.read_exact(&mut buf)?;
        let mut buf2: Vec<u16> = Vec::with_capacity(count / 2);
        for (i, val) in buf.iter().enumerate() {
            if i % 2 == 0 {
                buf2.push(*val as u16);
            } else {
                buf2[i / 2] += (*val as u16) << 8;
            }
        }
        Ok(buf2)
    }

    fn read_global_bits<const COUNT: usize>(&mut self, offset: u64) -> Result<[u8; COUNT]> {
        self.memory.seek(SeekFrom::Start(offset))?;
        let mut buf = [0; COUNT];
        self.memory.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl MemReader for VanillaMemReader {
    fn chapter_complete(&mut self) -> Result<bool> {
        // Celeste.Instance.AutosplitterInfo.ChapterComplete
        return Ok(u8::from_le_bytes(self.read_bits(0x12)?) == 1);
    }

    fn level_name(&mut self) -> Result<String> {
        // Celeste.Instance.AutosplitterInfo.Level
        let address = u64::from_le_bytes(self.read_bits(0)?);
        let length = u32::from_le_bytes(self.read_global_bits(address + 0x10)?);
        Ok(String::from_utf16(&self.read_vec_global_bits(
            address + 0x14,
            2 * length as usize,
        )?)?)
    }

    fn area_id(&mut self) -> Result<i32> {
        // Celeste.Instance.AutosplitterInfo.Chapter
        Ok(i32::from_le_bytes(self.read_bits(0x8)?))
    }

    fn area_difficulty(&mut self) -> Result<i32> {
        // Celeste.Instance.AutosplitterInfo.Mode
        Ok(i32::from_le_bytes(self.read_bits(0xc)?))
    }

    fn chapter_started(&mut self) -> Result<bool> {
        // Celeste.Instance.AutosplitterInfo.ChapterStarted
        return Ok(u8::from_le_bytes(self.read_bits(0x11)?) == 1);
    }

    fn game_time(&mut self) -> Result<f64> {
        // Celeste.Instance.AutosplitterInfo.FileTime
        Ok(i64::from_le_bytes(self.read_bits(0x28)?) as f64 / 10000000.)
    }

    fn level_time(&mut self) -> Result<f64> {
        // Celeste.Instance.AutosplitterInfo.ChapterTime
        Ok(i64::from_le_bytes(self.read_bits(0x18)?) as f64 / 10000000.)
    }

    fn strawberries(&mut self) -> Result<u32> {
        // Celeste.Instance.AutosplitterInfo.FileStrawberries
        Ok(u32::from_le_bytes(self.read_bits(0x30)?))
    }

    fn cassettes(&mut self) -> Result<u32> {
        // Celeste.Instance.AutosplitterInfo.FileCassettes
        Ok(u32::from_le_bytes(self.read_bits(0x34)?))
    }

    fn chapter_cassette_collected(&mut self) -> Result<bool> {
        // Celeste.Instance.AutosplitterInfo.ChapterCassette
        Ok(u32::from_le_bytes(self.read_bits(0x24)?) == 1)
    }

    fn heart_gems(&mut self) -> Result<u32> {
        // Celeste.Instance.AutosplitterInfo.FileHearts
        Ok(u32::from_le_bytes(self.read_bits(0x38)?))
    }

    fn chapter_heart_collected(&mut self) -> Result<bool> {
        // Celeste.Instance.AutosplitterInfo.ChapterHeart
        Ok(u32::from_le_bytes(self.read_bits(0x28)?) == 1)
    }

    fn starting_new_file(&mut self) -> Result<bool> {
        // This one's gonna be annoying
        todo!()
    }
}
