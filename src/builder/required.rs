use super::{delegate_deref, delegates, ArgumentBuilder, Build};
use crate::tree::{ArgumentCommandNode, CommandNode, NodeId, TaggedCommandNode, TreeGraph};
use crate::{ArgumentType, CommandRequirement};

pub struct RequiredArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    inner: ArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>,
    name: S,
    argument_type: AT,
    suggestions_provider: Option<SP>,
}

impl<'t, CS, AT, SP, R, M, S, CR> Build for RequiredArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
    R: CommandRequirement<CS>,
    S: AsRef<str>,
{
    type Node = CommandNode<CS, AT, SP, R, M, S, CR>;

    fn build(self) -> NodeId {
        self.inner
            .build(TaggedCommandNode::Argument(ArgumentCommandNode::new(
                self.name,
                self.argument_type,
                self.suggestions_provider,
            )))
    }
}

impl<'t, CS, AT, SP, R, M, S, CR> RequiredArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    pub fn new(tree: &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>, name: S, argument_type: AT) -> Self
    where
        R: CommandRequirement<CS>,
    {
        Self {
            inner: ArgumentBuilder::new(tree),
            name,
            argument_type,
            suggestions_provider: None,
        }
    }

    pub fn suggests(mut self, provider: SP) -> Self {
        self.suggestions_provider = Some(provider);
        self
    }

    pub fn suggestions_provider(&self) -> Option<&SP> {
        self.suggestions_provider.as_ref()
    }

    pub fn argument_type(&self) -> &AT {
        &self.argument_type
    }

    pub fn name(&self) -> &S {
        &self.name
    }

    delegates!(inner);
}

delegate_deref!(RequiredArgumentBuilder, inner);
