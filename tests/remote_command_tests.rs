use remote_cmd::{OutputType, RemoteCommand};

#[tokio::test]
async fn test_remote_command_stdout_ping() -> Result<(), Box<dyn std::error::Error>> {
    // This requires having a test server running that expects a CommandRequest and responds accordingly.
    let remote_command = RemoteCommand::new("bash")
        .arg("-c")
        .arg("echo Hello $NAME@$HOST; ping 127.0.0.1 -c 5")
        .env("NAME", "grok")
        .env("HOST", "remote.server");
    let mut remote_child = remote_command.spawn("127.0.0.1:8081").await?;

    // Use variables to capture stdout
    let mut stdout_lines = Vec::new();

    // Read from stdout
    while let Some(stream_line) = remote_child.read_stdout().await? {
        if let OutputType::Stdout = stream_line.output_type {
            stdout_lines.push(stream_line.line);
        }
    }
    // Wait for the process to exit and check the exit code
    let status = remote_child.wait().await?;

    assert_eq!(status, 0, "The remote command did not exit successfully");

    // Check that the stdout contains the expected greeting
    assert!(
        stdout_lines
            .iter()
            .any(|line| line.contains("Hello grok@remote.server")),
        "stdout does not contain the expected greeting"
    );

    let stdout_content = stdout_lines.join("\n");
    // Count occurrences of "bytes from" in the stdout content.
    let bytes_from_count = stdout_content.matches("bytes from").count();

    // Assert that "bytes from" appears exactly 5 times.
    assert_eq!(
        bytes_from_count, 5,
        "Expected 'bytes from' to appear 5 times, found {} times",
        bytes_from_count
    );

    Ok(())
}

#[tokio::test]
async fn test_remote_command_stderr_ping() -> Result<(), Box<dyn std::error::Error>> {
    // This requires having a test server running that expects a CommandRequest and responds accordingly.
    let remote_command = RemoteCommand::new("bash").arg("-c").arg("ping 8000.2.99.2"); // Command that should produce an error
    let mut remote_child = remote_command.spawn("127.0.0.1:8081").await?;

    // Use variables to capture stderr outputs
    let mut stderr_lines = Vec::new();

    // Read from stderr
    while let Some(stream_line) = remote_child.read_stderr().await? {
        if let OutputType::Stderr = stream_line.output_type {
            stderr_lines.push(stream_line.line);
        }
    }

    // Wait for the process to exit and check the exit code
    let status = remote_child.wait().await?;

    assert_ne!(status, 0, "Process exited with error: {}", status);

    // Check that the stdout contains the expected error
    assert!(
        stderr_lines
            .iter()
            .any(|line| line.contains("Name or service not known")),
        "stderr does not contain expected error"
    );

    Ok(())
}
