use anyhow::Result;
use procfs::process::{MMPermissions, MemoryMap, Process};
use std::{
    error::Error, fmt, fs::File, io::{Read, Seek, SeekFrom}
};

pub struct EverestMemReader {
    process: Process,
    map: MemoryMap,
    memory: File
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
                        match memory.read_exact(&mut buf) {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                        if buf.iter().eq(CORE_AUTOSPLITTER_MAGIC) {
                            let mut buf2: [u8; 1] = [0];
                            memory.seek(SeekFrom::Current(0x03))?;
                            match memory.read_exact(&mut buf2) {
                                Ok(_) => {}
                                Err(_) => {}
                            }
                            if u8::from_be_bytes(buf2) < CORE_AUTOSPLITTER_INFO_MIN_VERSION {
                                // println!("Bruh : {:?}", buf2);
                                continue;
                            }
                            let mem = process.mem()?;
                            return Ok(Self { process, map, memory });
                        }
                    }
                }
            }
        }
        Err(NotFoundError.into())
    }

    pub fn level_name(&mut self) -> Result<String> {
        self.memory.seek(SeekFrom::Start(self.map.address.0 + 0x38)).unwrap();
        let mut buf: [u8; 8] = [0u8; 8];
        match self.memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        let level_name_str = u64::from_le_bytes(buf);
        //println!("{:?}", level_name_str);
        let mut buf: [u8; 2] = [0; 2];
        self.memory.seek(SeekFrom::Start(level_name_str - 2)).unwrap();
        match self.memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        let name_len: u16 = u16::from_le_bytes(buf);
        let mut buf: Vec<u8> = vec![0; name_len as usize];
        self.memory.seek(SeekFrom::Start(level_name_str)).unwrap();
        match self.memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        return Ok(String::from_utf8(buf)?);
    }
}
