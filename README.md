
# Design Decisions
## Protcol
For communication between client and server, I decided to use JSON for protocol serialization with serde. This provides debugging clarity and allows users to easily serialize and deserialize messages. This project implements the following communication protocol:
```
pub enum OutputType {
    Stdout,
    Stderr,
    Exit,
}
pub struct CommandRequest {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
}

pub struct StreamLine {
    pub line: String,
    pub output_type: OutputType,
    pub exit_code: Option<i32>,
}
```

This structure allows for a flexible representation of command execution requests and their outputs. The CommandRequest struct captures all necessary information to execute a command on the server, including the program name, optional arguments, and environment variables. The StreamLine struct is designed to encapsulate the output of the executed command, distinguishing between standard output, standard error, and exit codes

## Remote Command
###  `Struct remote_cmd::RemoteCommand`
```pub struct RemoteCommand { /* private fields */ }```

### Implementations
`pub fn new(program: &str) -> RemoteCommand`
> Constructs a new RemoteCommand for launching the program at path `program`, with default configuration:
> * No arguments to the program
> * Inherit the remote process’s environment
> * Inherit the current process’s working directory


`pub fn arg(mut self, arg: &str) -> RemoteCommand `
> Adds an argument to pass to the program. These arguments are stackable.


`pub fn env(mut self, key: &str, value: &str) -> Self `
> Inserts or updates an environment variable. This will replace the remote process environment.

`pub async fn spawn(self, address: &str) -> Result<RemoteProcess, Box<dyn std::error::Error>>`
> Spawns a process on a remote machine at `address`. This function initiates a connection to the specified address, serializes the command request into JSON format, and sends it to the remote server. The server is expected to execute the command as specified by the CommandRequest struct, capturing the command's output and exit status. The spawn function awaits the establishment of the connection, the sending of the command request, and the initialization of a `RemoteProcess` object (which is analogous to a `std::process::Child`)


## Remote Process
###  `Struct remote_cmd::RemoteProcess`
```pub struct RemoteCommand { /* private fields */ }```


RemoteProcess provides methods to interact with the process running on the remote server. Under the hood, RemoteProcess utilizes tokio's multi-producer, single-consumer (mpsc) channels for stdout, stderr, and exit signaling. Since the stdout, stderr, and exit codes of the remote process are multiplexed and sent down a single socket, we need a way to demultiplex these messages so users can consume output types of their choosing. I chose to use mpsc channels because of their asynchronous design, which allows for non-blocking reads and writes. Standard buffers or deque do not inherently support asynchronous operations, which means I'd need additional mechanisms to prevent blocking behavior, complicating the design. RemoteProcess spawns a new asynchronous task to read from the TcpStream, process each `StreamLine`, and dispatch them to their respective channels. 


## Implementation
`pub async fn read_stdout(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>>`
> Asynchronously reads a line from the standard output of the remote process. If the process has closed its standard output, resulting in no more lines to read, the function returns None. Otherwise, it returns the line wrapped in Some.

`pub async fn read_stderr(&mut self) -> Result<Option<StreamLine>, Box<dyn std::error::Error>>`
> Similar to read_stdout, but reads from the standard error stream.

`pub async fn wait(&mut self) -> Result<i32, Box<dyn std::error::Error>>`
> Waits for the remote process to exit and returns its exit code.
- - - -
## Server Overview
* **Command Line Interface**: Utilizes clap for parsing command-line arguments, specifically the IP address and port on which the server should listen.
* **TCP Server Setup**: Establishes a TcpListener to asynchronously accept incoming connections. For each connection, it spawns a new thread to handle the communication. I use TCP instead of UDP here because TCP ensures ordered, error-free, packet delivery. I also considered using gRPC or HTTP, but there's too much overhead with these approaches which would increase latency and complicate the code. 
* **Command Execution**: Upon receiving a command request (serialized as JSON), it deserializes the request into a CommandRequest struct, executes the specified command using `std::process::Command`, and captures the command's stdout and stderr streams.
* **Streaming Output**: Streams the command's output back to the client in real-time, wrapping each line of output in a StreamLine struct (also serialized as JSON). This includes distinguishing between stdout and stderr, as well as sending a final message indicating the command's exit status.
- - - -
## Example Usage
```
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

```

To run the above example, first, ensure that the server is listening on the remote host `cargo run --bin server 0.0.0.0 8081`. Then run `cargo run --example client`.
```
> Hello grok@remote.server !
> PING 127.0.0.1 (127.0.0.1) 56(84) bytes of data.
> 64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.015 ms
> 64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.040 ms
> 64 bytes from 127.0.0.1: icmp_seq=3 ttl=64 time=0.036 ms
> 64 bytes from 127.0.0.1: icmp_seq=4 ttl=64 time=0.041 ms
> 64 bytes from 127.0.0.1: icmp_seq=5 ttl=64 time=0.037 ms
> 
> --- 127.0.0.1 ping statistics ---
> 5 packets transmitted, 5 received, 0% packet loss, time 4094ms
> rtt min/avg/max/mdev = 0.015/0.033/0.041/0.009 ms
err> ping: 1234.56.7.8: Name or service not known
Process exited with code 2
```

This example above highlights the use of arguments, environment variables, line-by-line streaming, and exit codes.
