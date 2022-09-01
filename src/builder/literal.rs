use super::{delegate_deref, delegates, ArgumentBuilder, Build};
use crate::tree::{CommandNode, LiteralCommandNode, NodeId, TaggedCommandNode, TreeGraph};
use crate::{ArgumentType, CommandRequirement};

pub struct LiteralArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    inner: ArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>,
    literal: S,
}

impl<'t, CS, AT, SP, R, M, S, CR> Build for LiteralArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
    R: CommandRequirement<CS>,
    S: AsRef<str>,
{
    type Node = CommandNode<CS, AT, SP, R, M, S, CR>;

    /// See [LiteralArgumentBuilder::build]
    ///
    /// [LiteralArgumentBuilder::build]: https://github.com/Mojang/brigadier/blob/cf754c4ef654160dca946889c11941634c5db3d5/src/main/java/com/mojang/brigadier/builder/LiteralArgumentBuilder.java#L30-L38
    fn build(self) -> NodeId {
        self.inner
            .build(TaggedCommandNode::Literal(LiteralCommandNode::new(
                self.literal,
            )))
    }
}

impl<'t, CS, AT, SP, R, M, S, CR> LiteralArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    pub fn new(tree: &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>, literal: S) -> Self
    where
        R: CommandRequirement<CS>,
    {
        Self {
            inner: ArgumentBuilder::new(tree),
            literal,
        }
    }

    pub fn literal(&self) -> &S {
        &self.literal
    }

    delegates!(inner);
}

delegate_deref!(LiteralArgumentBuilder, inner);
