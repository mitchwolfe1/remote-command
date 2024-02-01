use serde_json::Value;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;
use std::process::Stdio;
use std::thread;

fn main() {
    let listener = start_server("0.0.0.0", 8080);
    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(stream) => {
                println!("New connection from {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
                    handle_connection(stream.try_clone().expect("couldn't clone stream"))
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    //execute_command("traceroute", &["8.8.8.8"]).unwrap();
}

fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);

    let mut cmd_buf = String::new();
    reader
        .read_line(&mut cmd_buf)
        .expect("Failed to read from stream");
    let json_obj = deserialize(cmd_buf);
    let cmd = json_obj["cmd"].as_str().unwrap();
    let args = json_obj["args"].as_str().unwrap();
    let splitted_args: Vec<&str> = args.split(' ').collect();
    let slice_ref: &[&str] = &splitted_args;

    execute_command(cmd, slice_ref, stream)
}

fn deserialize(data: String) -> Value {
    serde_json::from_str(&data).expect("couldn't deserialize")
}

fn start_server(ip_addr: &str, port: u16) -> TcpListener {
    let addr = format!("{}:{}", ip_addr, port);
    let listener = TcpListener::bind(&addr);
    match listener {
        Ok(listener) => {
            println!("Server listening on {}", &addr);
            listener
        }
        Err(e) => panic!("Cannot bind to {}, \n{}", &addr, e),
    }
}

fn execute_command(cmd_str: &str, arg_str: &[&str], mut stream: TcpStream) {
    let mut child = Command::new(cmd_str)
        .args(arg_str)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let reader = io::BufReader::new(stdout);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let line = format!("{}\n", line);

                stream
                    .write_all(line.as_bytes())
                    .expect("Could not write to stream")
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    let _ = child.wait().unwrap();
}
