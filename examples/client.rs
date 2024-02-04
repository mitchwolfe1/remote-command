use remote_cmd::RemoteCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote_command = RemoteCommand::new("bash")
        .arg("-c")
        .arg("echo Hello $NAME@$HOST !; ping 127.0.0.1 -c 5; ping 1234.56.7.8")
        .env("NAME", "grok")
        .env("HOST", "remote.server");

    let mut remote_child = remote_command.spawn("127.0.0.1:8081").await?;

    while let Some(stream_line) = remote_child.read_stdout().await? {
        println!("> {}", stream_line.line);
    }

    while let Some(stream_line) = remote_child.read_stderr().await? {
        println!("err> {}", stream_line.line);
    }

    let status = remote_child.wait().await?;
    println!("Process exited with code {}", status);

    Ok(())
}
