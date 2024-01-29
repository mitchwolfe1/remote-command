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
