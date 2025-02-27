use nix::{sys::stat::Mode, unistd::mkfifo};
use std::{
    fs::OpenOptions,
    io::{self, BufReader, BufRead},
    path::Path,
    thread,
    time::Duration,
};
#[warn(unused_mut)]
fn main() -> io::Result<()> {
    // pipe path
    let fifo_path = Path::new("/tmp/pipe");

    // Readonly
    let file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(&fifo_path);

    //detect
    //There is possible race condition. However, mkfifo is an atomic action, which means it would no race but return a EEXIST
    let file = match file {
        Ok(f) => f,
        Err(e) => {
            println!("Open FIFO for reading failed: {e}. Try to create FIFO.");
            match mkfifo(fifo_path, Mode::from_bits_truncate(0o666)) {
                Ok(_) => {
                    println!("FIFO created: {:?}", fifo_path);
                    OpenOptions::new()
                        .read(true)
                        .write(false)
                        .create(false)
                        .open(&fifo_path)
                        .expect("Failed to open FIFO after creation")
                }
                Err(e) => {
                    panic!("Failed to create FIFO: {e}");
                }
            }
        }
    };

    let mut reader = BufReader::new(file);

    println!("FIFO for reading is open. Waiting for data...");
    
    for line in reader.lines() {
        match line {
            Ok(l) => {
                println!("Received: {}", l);
            }
            Err(e) => {
                println!("Error reading FIFO: {e}");
                break;
            }
        }
    }
    Ok(())
}
