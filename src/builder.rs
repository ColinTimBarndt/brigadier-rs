mod literal;
mod required;

use crate::tree::{CommandNode, NodeId, TaggedCommandNode, TreeGraph};
use crate::{ArgumentType, Command, CommandRequirement, RedirectModifier};
use std::rc::Rc;

pub use literal::*;
pub use required::*;

#[must_use]
pub struct ArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    tree: &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>,
    arguments: Vec<NodeId>,
    command: Option<Rc<dyn Command<CS, AT::Value, M, Result = CR>>>,
    requirement: R,
    target: Option<NodeId>,
    modifier: Option<M>,
    forks: bool,
}

pub trait Build {
    type Node;
    fn build(self) -> NodeId;
}

pub trait BuilderFactory<'b, CS: 'b, AT: 'b, SP: 'b, R: 'b, M: 'b, S: 'b, CR: 'b>
where
    Self: FnOnce(&'b mut TreeGraph<CS, AT, SP, R, M, S, CR>) -> Self::Builder,
    AT: ArgumentType,
{
    type Builder: Build<Node = CommandNode<CS, AT, SP, R, M, S, CR>> + 'b;
}

impl<'b, F, B, CS: 'b, AT: 'b, SP: 'b, R: 'b, M: 'b, S: 'b, CR: 'b>
    BuilderFactory<'b, CS, AT, SP, R, M, S, CR> for F
where
    F: FnOnce(&'b mut TreeGraph<CS, AT, SP, R, M, S, CR>) -> B,
    B: Build<Node = CommandNode<CS, AT, SP, R, M, S, CR>> + 'b,
    AT: ArgumentType,
{
    type Builder = B;
}

// https://github.com/rust-lang/rust/issues/8995#issuecomment-692386147
// inherent associated types would be especially useful here

impl<'t, CS, AT, SP, R, M, S, CR> ArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    fn new(tree: &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>) -> Self
    where
        R: CommandRequirement<CS>,
    {
        Self {
            tree,
            arguments: vec![],
            command: None,
            requirement: R::always(),
            target: None,
            modifier: None,
            forks: false,
        }
    }

    pub fn then(&mut self, next: NodeId) {
        assert!(self.tree.contains_node(next));
        self.arguments.push(next);
    }

    #[inline(always)]
    pub fn then_build<FB, F>(&mut self, builder: FB, scope: F)
    where
        FB: for<'b> BuilderFactory<'b, CS, AT, SP, R, M, S, CR>,
        F: for<'r> FnOnce(&mut <FB as BuilderFactory<'r, CS, AT, SP, R, M, S, CR>>::Builder),
    {
        let node = {
            let mut b = builder(self.tree);
            scope(&mut b);
            b.build()
        };
        self.then(node);
    }

    /// All further arguments will be inherited from the target node.
    pub fn forward(&mut self, target: NodeId, modifier: Option<M>, fork: bool) {
        assert!(
            !self.arguments.is_empty(),
            "Cannot forward a node with children"
        );
        assert!(self.tree.contains_node(target));
        self.target = Some(target);
        self.modifier = modifier;
        self.forks = fork;
    }

    #[inline(always)]
    pub fn redirect(&mut self, target: NodeId) {
        self.forward(target, None, false)
    }

    #[inline(always)]
    pub fn redirect_modifier(&mut self, target: NodeId, modifier: M) {
        self.forward(target, Some(modifier), false);
    }

    #[inline(always)]
    pub fn fork(&mut self, target: NodeId, modifier: Option<M>) {
        self.forward(target, modifier, true);
    }

    pub fn redirect_id(&self) -> Option<NodeId> {
        self.target
    }

    pub fn redirect_ref(&self) -> Option<&CommandNode<CS, AT, SP, R, M, S, CR>> {
        self.target.map(|id| self.tree.get(id).unwrap())
    }

    pub fn tree(&mut self) -> &mut TreeGraph<CS, AT, SP, R, M, S, CR> {
        self.tree
    }

    pub fn forks(&self) -> bool {
        self.forks
    }

    fn build(self, tagged: TaggedCommandNode<AT, SP, S>) -> NodeId
    where
        R: CommandRequirement<CS>,
        S: AsRef<str>,
    {
        let node = CommandNode {
            id: Default::default(),
            edges: Default::default(),
            tagged,
            requirement: self.requirement,
            redirect: self.target,
            modifier: self.modifier,
            forks: self.forks,
            command: self.command,
        };
        let node_id = self.tree.insert(node);
        for argument_id in self.arguments {
            self.tree.add_child(node_id, argument_id);
        }
        node_id
    }
}

macro_rules! delegates {
    ($del:ident) => {
        #[inline(always)]
        pub fn then(&mut self, next: NodeId) -> &mut Self {
            self.$del.then(next);
            self
        }

        //#[inline(always)]
        //pub fn then_build<FB, F>(&mut self, builder: FB, scope: F) -> &mut Self
        //where
        //    FB: for<'b> $crate::builder::BuilderFactory<'b, CS, AT, SP, R, M, S, CR>,
        //    F: for<'b> FnOnce(
        //        &mut <FB as $crate::builder::BuilderFactory<'b, CS, AT, SP, R, M, S, CR>>::Builder,
        //    ),
        //{
        //    self.$del.then_build(builder, scope);
        //    self
        //}

        #[inline(always)]
        pub fn forward(&mut self, target: NodeId, modifier: Option<M>, fork: bool) -> &mut Self {
            self.$del.forward(target, modifier, fork);
            self
        }

        #[inline(always)]
        pub fn redirect(&mut self, target: NodeId) -> &mut Self {
            self.$del.redirect(target);
            self
        }

        #[inline(always)]
        pub fn redirect_modifier(&mut self, target: NodeId, modifier: M) -> &mut Self {
            self.$del.redirect_modifier(target, modifier);
            self
        }
    };
}

pub(self) use delegates;

macro_rules! delegate_deref {
    ($Builder:ident, $del:ident) => {
        impl<'t, CS, AT, SP, R, M, S, CR> ::std::ops::Deref
            for $Builder<'t, CS, AT, SP, R, M, S, CR>
        where
            AT: ArgumentType,
        {
            type Target = $crate::builder::ArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<'t, CS, AT, SP, R, M, S, CR> ::std::ops::DerefMut
            for $Builder<'t, CS, AT, SP, R, M, S, CR>
        where
            AT: ArgumentType,
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
    };
}

pub(self) use delegate_deref;

pub mod builders {
    use super::*;
    use literal::LiteralArgumentBuilder;

    #[macro_export]
    macro_rules! build_literal {
        ($parent:expr, $literal:expr, $builder:ident => $scope:block) => {
            $parent.then_build(
                $crate::builder::builders::literal($literal),
                |$builder: &mut _| $scope,
            )
        };
    }

    pub use build_literal;

    /// Convenience function for creating a [LiteralArgumentBuilder] using `then_build`.
    #[inline(always)]
    pub fn literal<CS, AT, SP, R, M, S, CR>(
        literal: S,
    ) -> impl for<'t> FnOnce(
        &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>,
    ) -> LiteralArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
    where
        AT: ArgumentType,
        M: RedirectModifier<CS, AT::Value>,
        R: CommandRequirement<CS>,
    {
        |tree| LiteralArgumentBuilder::new(tree, literal)
    }

    /// Convenience function for creating a [RequiredArgumentBuilder] using `then_build`.
    #[inline(always)]
    pub fn argument<'t, CS, AT, SP, R, M, S, CR>(
        name: S,
        argument_type: AT,
    ) -> impl FnOnce(
        &'t mut TreeGraph<CS, AT, SP, R, M, S, CR>,
    ) -> RequiredArgumentBuilder<'t, CS, AT, SP, R, M, S, CR>
    where
        AT: ArgumentType,
        M: RedirectModifier<CS, AT::Value>,
        R: CommandRequirement<CS>,
    {
        |tree| RequiredArgumentBuilder::new(tree, name, argument_type)
    }
}
