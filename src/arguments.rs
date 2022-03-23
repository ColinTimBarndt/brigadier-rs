use std::ops::{RangeFrom, RangeFull, RangeInclusive, RangeToInclusive};

use crate::{
    context::CommandContext,
    errors::{CommandErrorType, CommandSyntaxError},
    suggestion::{Suggestions, SuggestionsBuilder},
    CommandSource, StringReader,
};

#[async_trait::async_trait]
pub trait ArgumentType<'i, S>
where
    S: CommandSource,
{
    type Output;
    fn parse(&self, reader: &mut StringReader<'i>) -> Result<Self::Output, CommandSyntaxError<'i>>;
    async fn list_suggestions<'t, 'm>(
        _context: &CommandContext<'i, S>,
        _builder: SuggestionsBuilder<'i, 't, 'm>,
    ) -> Suggestions<'t, 'm> {
        Suggestions::EMPTY
    }
    fn examples(&self) -> &'static [&'static str] {
        &[]
    }
}

pub struct BoolArgumentType;

#[async_trait::async_trait]
impl<'i, S> ArgumentType<'i, S> for BoolArgumentType
where
    S: CommandSource,
{
    type Output = bool;
    fn parse(&self, reader: &mut StringReader<'i>) -> Result<bool, CommandSyntaxError<'i>> {
        reader.read_boolean()
    }
    async fn list_suggestions<'t, 'm>(
        _context: &CommandContext<'i, S>,
        mut builder: SuggestionsBuilder<'i, 't, 'm>,
    ) -> Suggestions<'t, 'm> {
        if "true".starts_with(builder.remaining_lower_case()) {
            builder.suggest_text("true");
        }
        if "false".starts_with(builder.remaining_lower_case()) {
            builder.suggest_text("false");
        }
        builder.build()
    }
    fn examples(&self) -> &'static [&'static str] {
        &["true", "false"]
    }
}

pub trait NumericArgumentBounds<T> {
    fn inclusive_minimum(&self) -> T;
    fn inclusive_maximum(&self) -> T;
    fn as_inclusive_range(&self) -> RangeInclusive<T> {
        self.inclusive_minimum()..=self.inclusive_maximum()
    }
}

macro_rules! impl_numeric_argument_bounds {
    ($t:ident, $T:ty) => {
        impl NumericArgumentBounds<$T> for RangeInclusive<$T> {
            #[inline]
            fn inclusive_minimum(&self) -> $T {
                *self.start()
            }
            #[inline]
            fn inclusive_maximum(&self) -> $T {
                *self.end()
            }
            #[inline]
            fn as_inclusive_range(&self) -> RangeInclusive<$T> {
                self.clone()
            }
        }

        impl NumericArgumentBounds<$T> for RangeFrom<$T> {
            #[inline]
            fn inclusive_minimum(&self) -> $T {
                self.start
            }
            #[inline]
            fn inclusive_maximum(&self) -> $T {
                $t::MAX
            }
        }

        impl NumericArgumentBounds<$T> for RangeToInclusive<$T> {
            #[inline]
            fn inclusive_minimum(&self) -> $T {
                $t::MIN
            }
            #[inline]
            fn inclusive_maximum(&self) -> $T {
                self.end
            }
        }

        impl NumericArgumentBounds<$T> for RangeFull {
            #[inline]
            fn inclusive_minimum(&self) -> $T {
                $t::MIN
            }
            #[inline]
            fn inclusive_maximum(&self) -> $T {
                $t::MAX
            }
        }
    };
}

impl_numeric_argument_bounds!(u8, u8);
impl_numeric_argument_bounds!(i8, i8);
impl_numeric_argument_bounds!(u16, u16);
impl_numeric_argument_bounds!(i16, i16);
impl_numeric_argument_bounds!(u32, u32);
impl_numeric_argument_bounds!(i32, i32);
impl_numeric_argument_bounds!(u64, u64);
impl_numeric_argument_bounds!(i64, i64);
impl_numeric_argument_bounds!(f32, f32);
impl_numeric_argument_bounds!(f64, f64);

pub struct NumericArgumentType<T> where RangeInclusive<T>: NumericArgumentBounds<T> {
   pub range: RangeInclusive<T>,
}

impl<T> NumericArgumentType<T> where RangeInclusive<T>: NumericArgumentBounds<T> {
   pub fn new(bounds: impl NumericArgumentBounds<T>) -> Self {
       Self {
           range: bounds.as_inclusive_range(),
       }
   }
}

macro_rules! impl_numeric_argument_type {
    ($Name:ident, $T:ty, $read:ident, $ErrTooSmall:ident, $ErrTooBig:ident) => {
        pub type $Name = NumericArgumentType<$T>;
        
        #[async_trait::async_trait]
        impl<'i, S> ArgumentType<'i, S> for $Name
        where
            S: CommandSource,
        {
            type Output = $T;
            fn parse(&self, reader: &mut StringReader<'i>) -> Result<$T, CommandSyntaxError<'i>> {
                let start = reader.cursor();
                let result = reader.$read()?;
                if result < *self.range.start() {
                   reader.set_cursor(start);
                   return Err(CommandSyntaxError::with_context(
                       CommandErrorType::$ErrTooSmall {
                           found: result,
                           min: *self.range.start(),
                       },
                       reader.context(),
                   ));
                }
                if result > *self.range.end() {
                   reader.set_cursor(start);
                   return Err(CommandSyntaxError::with_context(
                       CommandErrorType::$ErrTooBig {
                           found: result,
                           max: *self.range.end(),
                       },
                       reader.context(),
                   ));
                }
                Ok(result)
            }
        }
    };
}

impl_numeric_argument_type!(DoubleArgumentType, f64, read_double, DoubleTooSmall, DoubleTooBig);
