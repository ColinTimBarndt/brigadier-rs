use std::{rc::Rc, borrow::Cow};

use crate::context::StringReaderContext;

pub static CONTEXT_AMOUNT: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSyntaxError<'i> {
    pub error_type: CommandErrorType<'i>,
    pub context: Option<StringReaderContext<'i>>,
}

impl<'i> CommandSyntaxError<'i> {
    pub fn new(error_type: CommandErrorType<'i>) -> Self {
        Self {
            error_type,
            context: None,
        }
    }
    pub fn with_context(error_type: CommandErrorType<'i>, context: StringReaderContext<'i>) -> Self {
        Self {
            error_type,
            context: Some(context),
        }
    }
    pub fn raw_message(&self) -> String {
        self.error_type.to_string()
    }
    pub fn context(&self) -> Option<String> {
        if let Some(StringReaderContext { input, cursor }) = self.context {
            let mut result = String::new();
            if cursor > CONTEXT_AMOUNT {
                result.push_str("...");
            }
            result.push_str(&input[0.max(cursor - CONTEXT_AMOUNT)..cursor]);
            result.push_str("<--[HERE]");
            Some(result)
        } else {
            None
        }
    }
}

impl std::fmt::Display for CommandSyntaxError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.context() {
            Some(context) => write!(
                f,
                "{message} at position {cursor}: {context}",
                message = self.error_type,
                cursor = self.context.unwrap().cursor,
            )
            .into(),
            None => write!(f, "{}", self.error_type),
        }
    }
}
impl std::error::Error for CommandSyntaxError<'_> {}

/// https://github.com/Mojang/brigadier/blob/master/src/main/java/com/mojang/brigadier/exceptions/BuiltInExceptions.java
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum CommandErrorType<'i> {
    #[error("Double must not be less than {min}, found {found}")]
    DoubleTooSmall { found: f64, min: f64 },
    #[error("Double must not be more than {max}, found {found}")]
    DoubleTooBig { found: f64, max: f64 },

    #[error("Float must not be less than {min}, found {found}")]
    FloatTooSmall { found: f32, min: f32 },
    #[error("Float must not be more than {max}, found {found}")]
    FloatTooBig { found: f32, max: f32 },

    #[error("Integer must not be less than {min}, found {found}")]
    IntegerTooSmall { found: i32, min: i32 },
    #[error("Integer must not be more than {max}, found {found}")]
    IntegerTooBig { found: i32, max: i32 },

    #[error("Long must not be less than {min}, found {found}")]
    LongTooSmall { found: i64, min: i64 },
    #[error("Long must not be more than {max}, found {found}")]
    LongTooBig { found: i64, max: i64 },

    #[error("Expected literal {expected}")]
    LiteralIncorrect { expected: Rc<str> },

    #[error("Expected quote to start a string")]
    ReaderExpectedStartOfQuote,
    #[error("Unclosed quoted string")]
    ReaderExpectedEndOfQuote,
    #[error("Invalid escape sequence '{0}' in quoted string")]
    ReaderInvalidEscape(char),
    #[error("Invalid bool, expected true or false but found '{0}'")]
    ReaderInvalidBool(Cow<'i, str>),
    #[error("Expected bool")]
    ReaderExpectedBool,
    #[error("Invalid integer '{0}'")]
    ReaderInvalidInt(&'i str),
    #[error("Expected integer")]
    ReaderExpectedInt,
    #[error("Invalid long '{0}'")]
    ReaderInvalidLong(&'i str),
    #[error("Expected long")]
    ReaderExpectedLong,
    #[error("Invalid double '{0}'")]
    ReaderInvalidDouble(&'i str),
    #[error("Expected double")]
    ReaderExpectedDouble,
    #[error("Invalid float '{0}'")]
    ReaderInvalidFloat(&'i str),
    #[error("Expected float")]
    ReaderExpectedFloat,
    #[error("Expected '{0}'")]
    ReaderExpectedSymbol(String),

    #[error("Unknown command")]
    DispatcherUnknownCommand,
    #[error("Incorrect argument for command")]
    DispatcherUnknownArgument,
    #[error("Expected whitespace to end one argument, but found trailing data")]
    DispatcherExpectedArgumentSeparator,
    #[error("Could not parse command: {0}")]
    DispatcherParseException(String),
}
