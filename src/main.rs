use procfs::process::{MMPermissions, MemoryMap, Process};
use std::io::{Read, Seek, SeekFrom};
use std::io;
use std::{io::Write, thread::sleep, time::Duration};

fn get_process() -> Option<(Process, MemoryMap)> {
    const CORE_AUTOSPLITTER_MAGIC: &[u8] = b"EVERESTAUTOSPLIT\xF0\xF1\xF2\xF3";
    const CORE_AUTOSPLITTER_INFO_MIN_VERSION: u8 = 3;
    let all_processes: Vec<Process> = procfs::process::all_processes()
        .expect("Can't read /proc")
        .filter_map(|p| match p {
            Ok(p) => {
                if p.stat().ok()?.comm.contains("Celeste") {
                    // TODO: CHECK THIS IS RIGHT
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
                    memory.seek(SeekFrom::Start(map.address.0)).unwrap();
                    let mut buf: [u8; 20] = [0u8; 20];
                    match memory.read_exact(&mut buf) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    if buf.iter().eq(CORE_AUTOSPLITTER_MAGIC) {
                        let mut buf2: [u8; 1] = [0];
                        memory.seek(SeekFrom::Current(0x03)).unwrap();
                        match memory.read_exact(&mut buf2) {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                        if u8::from_be_bytes(buf2) < CORE_AUTOSPLITTER_INFO_MIN_VERSION {
                            println!("Bruh : {:?}", buf2);
                            continue;
                        }
                        return Some((process, map));
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn main() {
    let (process, map) = get_process().unwrap();
    let mut memory = process.mem().unwrap();
    loop {

        memory.seek(SeekFrom::Start(map.address.0 + 0x38)).unwrap();
        let mut buf: [u8; 8] = [0u8; 8];
        match memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        let level_name_str = u64::from_le_bytes(buf);
        //println!("{:?}", level_name_str);
        let mut buf: [u8; 2] = [0; 2];
        memory.seek(SeekFrom::Start(level_name_str - 2)).unwrap();
        match memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        let name_len: u16 = u16::from_le_bytes(buf);
        let mut buf: Vec<u8> = vec![0; name_len as usize];
        memory.seek(SeekFrom::Start(level_name_str)).unwrap();
        match memory.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        print!("\r\x1b[2K{:?}", str::from_utf8(&buf).unwrap());
        sleep(Duration::from_millis(10));
        io::stdout().flush().unwrap();
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    compile_error!(
        "This project is only made for Linux. Use the actual LiveSplit if you're on Windows!"
    );
}
