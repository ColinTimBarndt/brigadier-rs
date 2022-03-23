use std::borrow::Cow;

use nom::bytes::complete::take_while;

use crate::{errors::{CommandErrorType, CommandSyntaxError}, context::StringReaderContext};

const SYNTAX_ESCAPE: char = '\\';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringReader<'i> {
    input: &'i str,
    remaining: &'i str,
}

macro_rules! impl_read_number {
    ($fnname:ident, $num:ty, $err_enum:ident) => {
        pub fn $fnname(&mut self) -> Result<$num, CommandSyntaxError<'i>> {
            let (remaining, number) =
                take_while::<_, _, ()>(is_allowed_number)(self.remaining).unwrap();
            if number.is_empty() {
                return Err(CommandSyntaxError::new(CommandErrorType::ReaderExpectedInt));
            }
            match number.parse() {
                Ok(number) => {
                    self.remaining = remaining;
                    Ok(number)
                }
                Err(_) => Err(CommandSyntaxError::with_context(
                    CommandErrorType::$err_enum(number),
                    self.context(),
                )),
            }
        }
    };
}

impl<'i> StringReader<'i> {
    pub fn new(input: &'i str) -> Self {
        Self {
            input,
            remaining: input,
        }
    }

    #[inline]
    pub fn input(&self) -> &'i str {
        self.input
    }

    #[inline]
    pub fn remaining(&self) -> &'i str {
        self.remaining
    }

    #[inline]
    pub fn cursor(&self) -> usize {
        self.input.len() - self.remaining.len()
    }

    #[inline]
    pub fn set_cursor(&mut self, cursor: usize) {
        self.remaining = &self.input[cursor..];
    }

    pub fn context(&self) -> StringReaderContext<'i> {
        StringReaderContext {
            input: self.input,
            cursor: self.input.len() - self.remaining.len(),
        }
    }

    #[inline]
    pub fn skip(&mut self) {
        self.remaining = &self.remaining[1..];
    }

    #[inline]
    pub unsafe fn skip_unchecked(&mut self) {
        self.remaining = &self.remaining.get_unchecked(1..);
    }

    impl_read_number!(read_int, i32, ReaderInvalidInt);
    impl_read_number!(read_long, i64, ReaderInvalidInt);
    impl_read_number!(read_float, f32, ReaderInvalidInt);
    impl_read_number!(read_double, f64, ReaderInvalidInt);

    /// Reads a string (quoted or unquoted) with either the value `true` or `false` (case sensitive).
    pub fn read_boolean(&mut self) -> Result<bool, CommandSyntaxError<'i>> {
        let start = self.remaining;
        let value = self.read_string()?;
        if value == Cow::Borrowed("true") {
            return Ok(true);
        }
        if value == Cow::Borrowed("false") {
            return Ok(false);
        }
        self.remaining = start;
        Err(CommandSyntaxError::with_context(
            CommandErrorType::ReaderInvalidBool(value),
            self.context(),
        ))
    }

    /// Reads a simple, unquoted string without any escape sequences.
    pub fn read_unquoted_string(&mut self) -> Result<&'i str, CommandSyntaxError<'i>> {
        let (remaining, string) =
            take_while::<_, _, ()>(is_allowed_in_unquoted_string)(self.remaining).unwrap();
        self.remaining = remaining;
        Ok(string)
    }

    /// Reads a string surrounded by single or double quotes. Supports escape esquences
    /// `\\` and `\"` or `\'` (depends on the starting quote).
    pub fn read_quoted_string(&mut self) -> Result<Cow<'i, str>, CommandSyntaxError<'i>> {
        if self.remaining.len() == 0 {
            return Ok(Cow::Borrowed(""));
        }
        let quote = self.remaining.chars().next().unwrap();
        if !is_quoted_string_start(quote) {
            return Err(CommandSyntaxError::with_context(
                CommandErrorType::ReaderExpectedStartOfQuote,
                self.context(),
            ));
        }
        unsafe {
            // SAFETY: The length of self.remaining is >0
            self.skip_unchecked();
        }
        self.read_string_until(quote)
    }

    /// Reads a string that is either quoted or unquoted.
    pub fn read_string(&mut self) -> Result<Cow<'i, str>, CommandSyntaxError<'i>> {
        if self.remaining.len() == 0 {
            return Ok(Cow::Borrowed(""));
        }
        let quote = self.remaining.chars().next().unwrap();
        if is_quoted_string_start(quote) {
            unsafe {
                // SAFETY: The length of self.remaining is >0
                self.skip_unchecked();
            }
            self.read_string_until(quote)
        } else {
            self.read_unquoted_string().map(Cow::Borrowed)
        }
    }

    pub fn read_string_until(
        &mut self,
        terminator: char,
    ) -> Result<Cow<'i, str>, CommandSyntaxError<'i>> {
        // HACK loop as block because labels on blocks are unstable
        'read: loop {
            let len;
            let mut chars = self.remaining.char_indices();
            'borrowed: loop {
                // No need to allocate when nothing is escaped
                while let Some((idx, c)) = chars.next() {
                    if c == SYNTAX_ESCAPE {
                        len = idx;
                        break 'borrowed;
                    } else if c == terminator {
                        let result = &self.remaining[..idx];
                        self.remaining = &self.remaining[idx + 1..];
                        return Ok(Cow::Borrowed(result));
                    }
                }
                break 'read;
            }
            // Owned
            let mut result = String::from(&self.remaining[..len]);
            let mut escaped = true;
            while let Some((idx, c)) = chars.next() {
                if escaped {
                    if c == terminator || c == SYNTAX_ESCAPE {
                        result.push(c);
                        escaped = false;
                    } else {
                        self.remaining = &self.remaining[idx..];
                        return Err(CommandSyntaxError::with_context(
                            CommandErrorType::ReaderInvalidEscape(c),
                            self.context(),
                        ));
                    }
                } else if c == SYNTAX_ESCAPE {
                    escaped = true;
                } else if c == terminator {
                    self.remaining = &self.remaining[idx + 1..];
                    return Ok(Cow::Owned(result));
                } else {
                    result.push(c);
                }
            }
            break;
        }
        self.remaining = "";
        Err(CommandSyntaxError::with_context(
            CommandErrorType::ReaderExpectedEndOfQuote,
            self.context(),
        ))
    }

    pub fn skip_whitespace(&mut self) {
        let (remaining, _) = take_while::<_, _, ()>(is_java_space)(self.remaining).unwrap();
        self.remaining = remaining;
    }
}

fn is_allowed_number(c: char) -> bool {
    c >= '0' && c <= '9' || c == '.' || c == '-'
}

fn is_allowed_in_unquoted_string(c: char) -> bool {
    match c {
        '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '-' | '.' | '+' => true,
        _ => false,
    }
}

fn is_quoted_string_start(c: char) -> bool {
    c == '"' || c == '\''
}

/// https://docs.oracle.com/javase/8/docs/api/java/lang/Character.html#isWhitespace-int-
fn is_java_space(c: char) -> bool {
    match c {
        '\t'
        | '\n'..='\r'
        | '\u{001C}'..='\u{001F}'
        // Line Separator (Zl)
        | ' '
        | '\u{2028}'
        // Paragraph Separator (Zp)
        | '\u{2029}'
        // Space Separator (Zs)
        | '\u{1680}'
        | '\u{2000}'..='\u{2006}'
        | '\u{2008}'..='\u{200A}'
        | '\u{205F}'
        | '\u{3000}' => true,
        _ => false,
    }
}
