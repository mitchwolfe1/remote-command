use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OutputType {
    Stdout,
    Stderr,
    Exit,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRequest {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StreamLine {
    pub line: String,
    pub output_type: OutputType,
    pub is_final: bool,
    pub exit_code: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ProtocolError {
    InvalidCommand,
    ExecutionFailed { reason: String },
}
