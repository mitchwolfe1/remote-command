use serde_json::from_str;
use std::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::protocol::StreamLine;
use crate::OutputType;

pub struct RemoteProcess {
    stdout_rx: mpsc::Receiver<StreamLine>,
    stderr_rx: mpsc::Receiver<StreamLine>,
    exit_rx: mpsc::Receiver<i32>,
}

impl RemoteProcess {
    pub async fn new(stream: TcpStream) -> io::Result<Self> {
        // Create stdout/stderr/exit channels
        let (stdout_tx, stdout_rx) = mpsc::channel(100);
        let (stderr_tx, stderr_rx) = mpsc::channel(100);
        let (exit_tx, exit_rx) = mpsc::channel(1);

        // Wrap incoming stream with BufReader to parse line by line
        let stream_reader = BufReader::new(stream);

        // Create asyncrynous task to read incomming lines and dispatch to their respective channels
        tokio::spawn(async move {
            let mut lines = stream_reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                match from_str::<StreamLine>(&line) {
                    // deserializes StreamLine
                    Ok(stream_line) => match stream_line.output_type {
                        // match on output type and send to channel
                        OutputType::Stdout => {
                            if stdout_tx.send(stream_line).await.is_err() {
                                break;
                            }
                        }
                        OutputType::Stderr => {
                            if stderr_tx.send(stream_line).await.is_err() {
                                break;
                            }
                        }
                        OutputType::Exit => {
                            if let Some(exit_code) = stream_line.exit_code {
                                let _ = exit_tx.send(exit_code).await;
                                break;
                            }
                        }
                    },
                    Err(e) => eprintln!("Failed to deserialize StreamLine: {}", e),
                }
            }
        });

        Ok(Self {
            stdout_rx,
            stderr_rx,
            exit_rx,
        })
    }

    // Reads stdout channel
    pub async fn read_stdout(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
        Ok(self.stdout_rx.recv().await)
    }

    // Reads stderr channel
    pub async fn read_stderr(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
        Ok(self.stderr_rx.recv().await)
    }

    // Waits till remote process is finished with exit code.
    pub async fn wait(&mut self) -> Result<i32, Box<dyn std::error::Error>> {
        match self.exit_rx.recv().await {
            Some(exit_code) => Ok(exit_code),
            None => Err("Did not receive exit signal".into()),
        }
    }
}
