use clap::Parser;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::process::{Command, Stdio};
use std::thread;

#[derive(Parser)]
struct Cli {
    ip_addr: String,
    port: u16,
}
fn main() {
    let args = Cli::parse();
    let listener = start_server(&args.ip_addr, args.port).expect(&format!(
        "Failed to start server on {}:{}",
        &args.ip_addr, args.port
    ));
    println!("Server started on {}:{}", &args.ip_addr, args.port);
    for tcp_stream in listener.incoming() {
        match tcp_stream {
            Ok(stream) => {
                println!("New connection from {}", stream.peer_addr().unwrap());
                thread::spawn(move || handle_connection(stream));
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(&stream);

    let mut cmd_buf = String::new();
    reader
        .read_line(&mut cmd_buf)
        .expect("Failed to read from stream");
    let json_obj = deserialize(cmd_buf);
    let cmd = json_obj["cmd"].as_str().unwrap();
    let args = json_obj["args"].as_str().unwrap();
    //let env_vars = json_obj["env"].as_str().unwrap();
    let splitted_args: Vec<&str> = args.split(' ').collect();
    let slice_ref: &[&str] = &splitted_args;

    let env_vars: Option<Vec<(&str, &str)>> = Some(vec![("NAME", "grok")]);
    execute_command(cmd, slice_ref, &stream, env_vars)
}

fn deserialize(data: String) -> Value {
    println!("{}", &data);
    serde_json::from_str(&data).expect("couldn't deserialize")
}

fn start_server(ip_addr: &str, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{}", ip_addr, port);
    TcpListener::bind(&addr)
}

fn execute_command(
    cmd_str: &str,
    arg_str: &[&str],
    mut stream: &TcpStream,
    env_vars: Option<Vec<(&str, &str)>>,
) -> Result<()> {
    let command_line = format!("{} {}", cmd_str, arg_str.join(" "));

    let mut command = Command::new("sh");
    command.arg("-c").arg(&command_line);

    if let Some(vars) = env_vars {
        for (key, value) in vars {
            command.env(key, value);
        }
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(child) => child,
        Err(e) => {
            let line = format!("Failed to execute command: {}", e);
            stream.write_all(line.as_bytes());
            return Err(e.into());
        }
    };
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let stderr = child.stderr.take().expect("Failed to open stderr");
    stream_write_lines(stream, BufReader::new(stdout));
    stream_write_lines(stream, BufReader::new(stderr));

    // Wait for child process to complete
    child.wait()?;
    Ok(())
}

fn stream_write_lines<R: Read>(mut stream: &TcpStream, reader: BufReader<R>) -> Result<()> {
    for line in reader.lines() {
        let line = line?.add("\n");
        stream.write_all(line.as_bytes())?;
    }
    Ok(())
}
