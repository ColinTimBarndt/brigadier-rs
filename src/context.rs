use std::ops::Range;

use crate::{command::Command, tree::RedirectModifier};

pub type StringRange = Range<usize>;

pub struct CommandContext<'i, S> where S: Clone {
    pub source: S,
    pub input: &'i str,
    pub command: Command<'i, S>,
    pub arguments: (),
    pub root_node: (),
    pub nodes: (),
    pub range: StringRange,
    child: (),
    pub modifier: Option<RedirectModifier<'i, S>>,
    pub forks: (),
}

impl<'i, S> CommandContext<'i, S> where S: Clone {
    #[inline]
    pub fn has_nodes(&self) -> bool {
        //self.nodes
        todo!("CommandContext.has_nodes")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StringReaderContext<'i> {
    pub input: &'i str,
    pub cursor: usize,
}
