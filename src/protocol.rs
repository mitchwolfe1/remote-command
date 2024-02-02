use serde::{Deserialize, Serialize};

// TODO: TEST DEBUG

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRequest {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub env: Option<Vec<(String, String)>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutputType {
    Stdout,
    Stderr,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct StreamLine {
    pub line: String,
    pub output_type: OutputType,
    pub is_final: bool,
}
