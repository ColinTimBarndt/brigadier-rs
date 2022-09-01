use thiserror::Error;

#[derive(Debug, Error)]
#[error("Command Syntax Error")]
pub struct CommandSyntaxError;
