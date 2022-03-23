use crate::{context::CommandContext, errors::CommandSyntaxError};

// #[derive(Clone, Copy)]
// pub struct Command<'s, S: 's> {
//     pub(crate) single_success: i32,
//     pub(crate) function: &'s dyn for<'i> Fn(CommandContext<'s, 'i, S>) -> i32,
// }

// impl<'s, S: 's> Command<'s, S> {
//     pub fn run<'i>(&self, context: CommandContext<'s, 'i, S>) -> i32 {
//         (*self.function)(context)
//     }
// }

// impl<'s, 'i, S: 's, F> From<&'s F> for Command<'s, S>
// where
//     F: Fn(CommandContext<'s, 'i, S>) -> i32,
// {
//     fn from(f: &'s F) -> Self {
//         Self {
//             single_success: 1,
//             function: f as _,
//         }
//     }
// }

pub type Command<'i, S> = fn(&CommandContext<'i, S>) -> Result<i32, CommandSyntaxError<'i>>;