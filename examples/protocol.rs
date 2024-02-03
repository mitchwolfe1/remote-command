use serde_json::from_str;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

use remote_cmd::{CommandRequest, StreamLine};

async fn run_client() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let command_request = CommandRequest {
        program: "bash".to_string(),
        args: Some(vec!["-c".to_string(), "echo $NAME".to_string()]),
        env: Some(vec![("NAME".to_string(), "grok".to_string())]),
    };

    let request = serde_json::to_string(&command_request).unwrap();
    stream.write_all(request.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let reader = BufReader::new(stream);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let stream_line: StreamLine = from_str(&line).unwrap();
        println!("Received: {:?}", stream_line.line);

        if stream_line.is_final {
            break;
        }
    }

    Ok(())
}

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(run_client()).unwrap();
}
