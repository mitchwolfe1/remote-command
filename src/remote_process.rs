use serde_json::from_str;
use tokio::sync::mpsc;
use std::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use crate::OutputType;
use crate::protocol::StreamLine;

pub struct RemoteProcess {
    stdout_rx: mpsc::Receiver<StreamLine>,
    stderr_rx: mpsc::Receiver<StreamLine>,
}

impl RemoteProcess {
    pub async fn new(stream: TcpStream) -> io::Result<Self> {
        let (stdout_tx, stdout_rx) = mpsc::channel(100); 
        let (stderr_tx, stderr_rx) = mpsc::channel(100);

        let stream_reader = BufReader::new(stream);

        tokio::spawn(async move {
            let mut lines = stream_reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                match from_str::<StreamLine>(&line) {
                    Ok(stream_line) => {
                        match stream_line.output_type {
                            OutputType::Stdout => {
                                if stdout_tx.send(stream_line).await.is_err() {
                                    break; 
                                }
                            },
                            OutputType::Stderr => {
                                if stderr_tx.send(stream_line).await.is_err() {
                                    break;
                                }
                            },
                            _ => {}
                        }
                    },
                    Err(e) => eprintln!("Failed to deserialize StreamLine: {}", e),
                }
            }
        });

        Ok(Self { stdout_rx, stderr_rx })
    }



    pub async fn read_stdout(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
        Ok(self.stdout_rx.recv().await)
    }

    pub async fn read_stderr(&mut self) ->Result<Option<StreamLine>, Box<dyn std::error::Error>> {
        Ok(self.stderr_rx.recv().await)
    }

    // pub async fn read_line(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
    //     let mut line = String::new();
    //     match self.stream.read_line(&mut line).await {
    //         Ok(0) => Ok(None), // EOF
    //         Ok(_) => {
    //             let stream_line: StreamLine = from_str(&line.trim_end())
    //                 .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    //             Ok(Some(stream_line))
    //         }
    //         Err(e) => Err(Box::new(e)),
    //     }
    // }


    // pub async fn read_stdout(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
    //     while let Some(line) = self.read_line().await? {
    //         if line.output_type == OutputType::Stdout {
    //             return Ok(Some(line));
    //         }
    //     }
    //     Ok(None)
    // }

    // pub async fn read_stderr(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>> {
    //     while let Some(line) = self.read_line().await? {
    //         if line.output_type == OutputType::Stderr {
    //             return Ok(Some(line));
    //         }
    //     }
    //     Ok(None)
    // }

    pub async fn wait(&self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
