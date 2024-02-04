// Protocol that client and server must implement to interface with each other

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputType {
    Stdout,
    Stderr,
    Exit,
}
#[derive(Serialize, Deserialize)]
pub struct CommandRequest {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize)]
pub struct StreamLine {
    pub line: String,
    pub output_type: OutputType,
    pub exit_code: Option<i32>,
}
