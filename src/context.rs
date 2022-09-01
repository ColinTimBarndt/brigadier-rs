use core::fmt;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::ops::{Deref, Range};
use std::rc::Rc;

use crate::{ParsedValue, RedirectModifier};

/// See [CommandContext.java][src]
///
/// [src]: https://github.com/Mojang/brigadier/blob/master/src/main/java/com/mojang/brigadier/context/CommandContext.java
#[derive(Clone)]
pub struct CommandContext<'c, 'i, CS, PV, M>
where
    PV: ParsedValue,
{
    source: CS,
    input: &'i str,
    command: (),
    arguments: Rc<HashMap<String, ParsedArgument<PV>>>,
    root_node: (),
    nodes: (),
    range: Range<usize>,
    children: &'c [Self],
    modifier: Option<M>,
    forks: bool,
}

impl<'c, 'i, CS, PV, M> CommandContext<'c, 'i, CS, PV, M>
where
    PV: ParsedValue,
    M: RedirectModifier<CS, PV>,
{
    pub fn new(
        source: CS,
        input: &'i str,
        arguments: Rc<HashMap<String, ParsedArgument<PV>>>,
        command: (),
        root_node: (),
        nodes: (),
        range: Range<usize>,
        children: &'c [Self],
        modifier: Option<M>,
        forks: bool,
    ) -> Self {
        Self {
            source,
            input,
            command,
            arguments,
            root_node,
            nodes,
            range,
            children,
            modifier,
            forks,
        }
    }

    pub fn clone_for(&self, source: CS) -> Self
    where
        M: Clone,
    {
        Self {
            source,
            input: self.input,
            command: self.command,
            arguments: self.arguments.clone(),
            root_node: self.root_node,
            nodes: self.nodes,
            range: self.range.clone(),
            children: self.children,
            modifier: self.modifier.clone(),
            forks: self.forks,
        }
    }

    pub fn source(&self) -> &CS {
        &self.source
    }

    pub fn child(&self) -> Option<&'c Self> {
        self.children.get(0)
    }
}

/// See [ParsedArgument.java][src]
///
/// [src]: https://github.com/Mojang/brigadier/blob/master/src/main/java/com/mojang/brigadier/context/ParsedArgument.java
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ParsedArgument<PV> {
    range: Range<usize>,
    result: PV,
}

impl<PV> ParsedArgument<PV>
where
    PV: ParsedValue,
{
    pub fn new(start: usize, end: usize, result: PV) -> Self {
        Self {
            range: start..end,
            result,
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn result(&self) -> &PV {
        &self.result
    }
}

/// See [StringRange.java][src]
///
/// [src]: https://github.com/Mojang/brigadier/blob/master/src/main/java/com/mojang/brigadier/context/StringRange.java
#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct StringRange(Range<usize>);

impl StringRange {
    pub fn at(pos: usize) -> Self {
        Self(pos..pos)
    }

    pub fn between(start: usize, end: usize) -> Self {
        Self(start..end)
    }

    pub fn encompassing(a: StringRange, b: StringRange) -> Self {
        let min = usize::min(a.start, b.start);
        let max = usize::max(a.end, b.end);
        StringRange(min..max)
    }
}

impl Deref for StringRange {
    type Target = Range<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Range<usize>> for StringRange {
    fn from(range: Range<usize>) -> Self {
        Self(range)
    }
}

impl fmt::Debug for StringRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
