
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
* No arguments to the program
* Inherit the remote process’s environment
* Inherit the current process’s working directory


`pub fn arg(mut self, arg: &str) -> RemoteCommand `

> Adds an argument to pass to the program. These arguments are stackable.


`pub fn env(mut self, key: &str, value: &str) -> Self `

> Inserts or updates an environment variable. This will replace the remote process environment.

The library includes a RemoteCommand implementation. The `arg`, and `env` functions allow users to stack multiple arguments and environment variables. The `spawn(address: &str)` function returns a RemoteProcess, similar to a `std::process::Child`


`pub async fn spawn(self, address: &str) -> Result<RemoteProcess, Box<dyn std::error::Error>>`
> Spawns a process on a remote machine at `address`. This function initiates a connection to the specified address, serializes the command request into JSON format, and sends it to the remote server. The server is expected to execute the command as specified by the CommandRequest struct, capturing the command's output and exit status. The spawn function awaits the establishment of the connection, the sending of the command request, and the initialization of a `RemoteProcess` object (which is analogous to a `std::process::Child`)

- - - -

## Remote Process
###  `Struct remote_cmd::RemoteProcess`
```pub struct RemoteCommand { /* private fields */ }```








# Takehome Assignment


A common part of many standard libraries is an interface to create processes and observe their output, eg Rust's `std::process::Command` or Python's `subprocess.Popen`. For this project, we want to make a similar interface, but the process will run on a remote server.

Example usage:

```rust
use remote_cmd::RemoteCommand;


let proc = RemoteCommand::new("bash")
    .arg("-c")
    .arg("Hello $NAME from $HOST")
    .env("NAME", "Grok")
    .spawn("localhost:8080")
    .await?;

let mut stdout = proc.stdout.take().unwrap();
while let Some(line) = stdout.try_next().await? {
    println!("> {line}");
}

let status = proc.wait()?;
println!("Process exited with code {}", status.code().unwrap());
```

Example output:

```
> Hello Grok from My-MacBook-Pro.local
Process exited with code 0
```

It's alright if the interface deviates slightly from the above, but the key points are:

* Specify an arbitrary command w/ args
* Override environment variables
* Spawn the process given the address of a server
* Provide streaming stdout/stderr while the process runs (above example is a line-based stream)
* Return the exit status

Note: it is not necessary to implement the entire interface of `std::process::{Command, Child}`, just the few example methods above are sufficient.

Deliverables:
* A Rust crate consisting of:
    * `[lib]` that exposes the remote command interface
    * `[[bin]]` to run the server
* Brief explanation of any interesting design or library choices
