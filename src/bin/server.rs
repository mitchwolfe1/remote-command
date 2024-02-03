use clap::Parser;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Result, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::process::{Command, Stdio};
use std::thread;

use remote_cmd::{CommandRequest, OutputType, StreamLine};

// I should explore redirecting the childs stdout and stderr to the socket
// on the file descriptor level

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
    reader.read_line(&mut cmd_buf)?;

    let command_request: CommandRequest =
        serde_json::from_str(&cmd_buf).expect("Failed to deserialize command request");

    execute_command(&command_request, &stream)
}

fn start_server(ip_addr: &str, port: u16) -> Result<TcpListener> {
    let addr = format!("{}:{}", ip_addr, port);
    TcpListener::bind(&addr)
}

fn execute_command(command_request: &CommandRequest, mut stream: &TcpStream) -> Result<()> {
    let mut command = Command::new(&command_request.program);

    if let Some(args) = &command_request.args {
        command.args(args);
    }

    if let Some(env_vars) = &command_request.env {
        for (key, value) in env_vars {
            command.env(key, value);
        }
    }

    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdout) = child.stdout.take() {
        stream_write_lines(stream, BufReader::new(stdout), OutputType::Stdout)?;
    }

    if let Some(stderr) = child.stderr.take() {
        stream_write_lines(stream, BufReader::new(stderr), OutputType::Stderr)?;
    }

    // Wait for child process to complete
    let status = child.wait()?;

    let final_message = StreamLine {
        line: format!("{}", status),
        output_type: OutputType::Exit,
        is_final: true,
    };

    let final_json = serde_json::to_string(&final_message)?;
    stream.write_all(final_json.as_bytes())?;

    Ok(())
}

fn stream_write_lines<R: Read>(
    mut stream: &TcpStream,
    reader: BufReader<R>,
    output_type: OutputType,
) -> Result<()> {
    for line in reader.lines() {
        let message = StreamLine {
            line: line?,
            output_type: output_type.clone(),
            is_final: false,
        };

        let serialized = serde_json::to_string(&message)?;
        stream.write_all(serialized.as_bytes())?;
        stream.write_all(b"\n")?;
    }
    Ok(())
}
