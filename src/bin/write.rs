use nix::{sys::stat::Mode, unistd::mkfifo};
use std::{
    fs::OpenOptions,
    io::{self, BufWriter, Write, BufRead, Read},
    path::Path,
};
fn main() -> io::Result<()>{
    //pipe path
    let fifo_path = Path::new("/tmp/pipe");
    //open file with write only
    //This is sync operations, if no read -> blocking
    let file = OpenOptions::new()
    .read(false)
    .write(true)
    .create(false)
    .open(&fifo_path);
    let mut file = match file{
        Ok(f) => f,
        Err(e) => {
            println!("Open pipe failed: {e}, try to create FIFO");
            //FIFO create
            match mkfifo(fifo_path,Mode::from_bits_truncate(0o666)){
                Ok(_) => {
                    println!("FIFO created: {:?}", fifo_path);
                    //try open again
                    OpenOptions::new()
                    .read(false)
                    .write(true)
                    .create(false)
                    .open(&fifo_path)
                    .expect("Failed to open FIFO after creation")
                },
                Err(e) => {
                    panic!("Create FIFO failed:{e}");
                }
            }
        }
    };
    //set up writer
    let mut writer = BufWriter::new(file);
    //read user input case (2)
    println!("Pipe connected.Please select your input types: 1 -- Keyboard 2 -- File");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;
    let user_input = user_input.trim();

    match user_input{
        //case 1 : keyboard
        "1" => {
            println!("Enter your msg (Type \"exit\" to quit): ");
            let stdin = io::stdin();
            for line_result in stdin.lock().lines() {
                let line = line_result?;
                if line.trim() == "exit" {
                    println!("Pipe closed.");
                    break;
                }
                //broken pipe detect
                //userinput -> bytes -> FIFO
                if let Err(e) = writer.write_all(line.as_bytes()) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        println!("BrokenPipe detected! Check your reader.");
                        break;
                    } else {
                        println!("Oops, something wrong: {e}");
                    }
                }
                //line feed
                if let Err(e) = writer.write_all(b"\n") {
                    println!("Line feed failed: {e}");
                }
                //flush buffer
                if let Err(e) = writer.flush() {
                    println!("Refresh failed: {e}");
                }
            }
        }
        "2" => {
            println!("Enter the file path to transmit:");
            let mut file_path = String::new();
            io::stdin().read_line(&mut file_path)?;
            let file_path = file_path.trim();
            let mut file_send = match OpenOptions::new()
            .read(true)
            .open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to open file: {e}");
                    return Ok(());
                }
            };
            let mut file_content = String::new();
            if let Err(e) = file_send.read_to_string(&mut file_content) {
                println!("Failed to read file: {e}");
            } else {
                if let Err(e) = writer.write_all(file_content.as_bytes()) {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        println!("BrokenPipe detected! Check your reader.");
                    } else {
                        println!("Error writing file content: {e}");
                    }
                }
                if let Err(e) = writer.write_all(b"\n") {
                    println!("Line feed failed: {e}");
                }
                if let Err(e) = writer.flush() {
                    println!("Flush failed: {e}");
                }
                println!("File transmitted successfully.");
            }
        }
        _ => {
            println!("Invalid input type selected.");
        }
    }
    Ok(())
}