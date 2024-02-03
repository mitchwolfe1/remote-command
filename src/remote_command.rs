use crate::protocol::{CommandRequest};
use crate::remote_process::RemoteProcess;
use serde_json::{to_string};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncWriteExt};
use tokio::net::TcpStream;

pub struct RemoteCommand {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
}

impl RemoteCommand {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: None,
            env: None,
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        let args = self.args.get_or_insert_with(Vec::new);
        args.push(arg.to_string());
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        let envs = self.env.get_or_insert_with(Vec::new);
        envs.push((key.to_string(), value.to_string()));
        self
    }

    pub async fn spawn(self, address: &str) -> Result<RemoteProcess, Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(address).await?;

        let request = CommandRequest {
            program: self.program,
            args: self.args,
            env: self.env,
        };

        let request_json =
            to_string(&request).map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
        stream.write_all(request_json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        Ok(RemoteProcess::new(stream).await?)
    }
}
