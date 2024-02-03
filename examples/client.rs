use remote_cmd::RemoteCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote_command = RemoteCommand::new("bash")
        .arg("-c")
        .arg("echo Hello $NAME@$HOST; ping 8.8.88..8 -c 5")
        .env("NAME", "grok")
        .env("HOST", "remote.server");

    let mut remote_child = remote_command.spawn("127.0.0.1:8081").await?;

    while let Some(stream_line) = remote_child.read_stdout().await? {
        println!("stdout> {}", stream_line.line);
        if stream_line.is_final {
            break;
        }
    }

    while let Some(stream_line) = remote_child.read_stderr().await? {
        println!("stderr> {}", stream_line.line);
        if stream_line.is_final {
            break;
        }
    }

    let status = remote_child.wait().await?;
    println!("Process exited with code {}", status);

    Ok(())
}
