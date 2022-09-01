use std::borrow::Cow;
use std::fmt::Formatter;
use std::io::Write;
use std::rc::Rc;
use std::{fmt, io};

use indexmap::IndexMap;
use slotmap::{new_key_type, SlotMap};

use crate::{
    ArgumentType, Command, CommandContext, CommandRequirement, RedirectModifier, SuggestionProvider,
};

new_key_type! {
    pub struct NodeId;
}

pub struct TreeGraph<CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    root: NodeId,
    nodes: SlotMap<NodeId, CommandNode<CS, AT, SP, R, M, S, CR>>,
}

impl<CS, AT, SP, R, M, S, CR> TreeGraph<CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    pub fn new() -> Self
    where
        R: CommandRequirement<CS>,
    {
        let mut this = Self {
            root: NodeId::default(),
            nodes: SlotMap::with_key(),
        };
        this.root = this.insert(CommandNode {
            id: Default::default(),
            edges: Default::default(),
            tagged: TaggedCommandNode::Root(RootCommandNode),
            requirement: R::always(),
            redirect: None,
            modifier: None,
            forks: false,
            command: None,
        });
        this
    }

    pub fn root_id(&self) -> NodeId {
        self.root
    }

    pub fn contains_node(&self, node: NodeId) -> bool {
        self.nodes.contains_key(node)
    }

    pub fn get(&self, node: NodeId) -> Option<&CommandNode<CS, AT, SP, R, M, S, CR>> {
        self.nodes.get(node)
    }

    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut CommandNode<CS, AT, SP, R, M, S, CR>> {
        self.nodes.get_mut(node)
    }

    pub(crate) fn insert(&mut self, node: CommandNode<CS, AT, SP, R, M, S, CR>) -> NodeId
    where
        R: CommandRequirement<CS>,
    {
        let id = self.nodes.insert(node);
        let node = unsafe { self.nodes.get_unchecked_mut(id) };
        node.id = id;
        id
    }

    /// See [CommandNode::addChild]
    ///
    /// [addChild]: https://github.com/Mojang/brigadier/blob/cf754c4ef654160dca946889c11941634c5db3d5/src/main/java/com/mojang/brigadier/tree/CommandNode.java#L68-L90
    pub fn add_child(&mut self, parent_id: NodeId, node_id: NodeId)
    where
        S: AsRef<str>,
    {
        let [parent, node] = self.nodes.get_disjoint_mut([parent_id, node_id]).unwrap();
        if matches!(node.tagged, TaggedCommandNode::Root(_)) {
            panic!("Cannot add a RootCommandNode as a child to any other CommandNode");
        }
        let node_name = node.name();
        if let Some(&child_id) = parent.edges.children.get(node_name) {
            // We've found something to merge onto
            let mut node = self.nodes.remove(node_id).unwrap();
            let child = &mut self.nodes[child_id];
            if let Some(command) = node.command.take() {
                child.command = Some(command);
            }
            for &grandchild_id in node.children() {
                self.add_child(child_id, grandchild_id);
            }
        } else {
            parent.edges.children.insert(node_name.to_owned(), node_id);
            match &node.tagged {
                TaggedCommandNode::Root(_) => unreachable!(),
                TaggedCommandNode::Literal(_) => {
                    parent.edges.literal.insert(node_name.to_owned(), node_id);
                }
                TaggedCommandNode::Argument(_) => {
                    parent.edges.argument.insert(node_name.to_owned(), node_id);
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NodeChildren {
    children: IndexMap<String, NodeId>,
    literal: IndexMap<String, NodeId>,
    argument: IndexMap<String, NodeId>,
}

pub enum TaggedCommandNode<AT, SP, S> {
    Root(RootCommandNode),
    Literal(LiteralCommandNode<S>),
    Argument(ArgumentCommandNode<AT, SP, S>),
}

pub struct CommandNode<CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    pub(crate) id: NodeId,
    pub(crate) edges: NodeChildren,
    pub(crate) tagged: TaggedCommandNode<AT, SP, S>,
    pub(crate) requirement: R,
    pub(crate) redirect: Option<NodeId>,
    pub(crate) modifier: Option<M>,
    pub(crate) forks: bool,
    pub(crate) command: Option<Rc<dyn Command<CS, AT::Value, M, Result = CR>>>,
}

impl<CS, AT, SP, R, M, S, CR> CommandNode<CS, AT, SP, R, M, S, CR>
where
    AT: ArgumentType,
{
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn name(&self) -> &str
    where
        S: AsRef<str>,
    {
        match &self.tagged {
            TaggedCommandNode::Root(_) => "",
            TaggedCommandNode::Literal(l) => l.literal.as_ref(),
            TaggedCommandNode::Argument(a) => a.name.as_ref(),
        }
    }

    pub fn usage_text(&self) -> Cow<str>
    where
        S: AsRef<str>,
    {
        match &self.tagged {
            TaggedCommandNode::Root(_) => Cow::from(""),
            TaggedCommandNode::Literal(l) => Cow::from(l.literal.as_ref()),
            TaggedCommandNode::Argument(a) => Cow::from(format!("<{}>", a.name.as_ref())),
        }
    }

    pub fn write_usage_text(&self, w: &mut impl Write) -> io::Result<()>
    where
        S: AsRef<str>,
    {
        match &self.tagged {
            TaggedCommandNode::Root(_) => Ok(()),
            TaggedCommandNode::Literal(l) => write!(w, "{}", l.literal.as_ref()),
            TaggedCommandNode::Argument(a) => write!(w, "<{}>", a.name.as_ref()),
        }
    }

    pub fn command(&self) -> Option<&dyn Command<CS, AT::Value, M, Result = CR>> {
        self.command.as_ref().map(|cmd| &**cmd)
    }

    pub fn redirect(&self) -> Option<NodeId> {
        self.redirect
    }

    pub fn modifier(&self) -> Option<&M> {
        self.modifier.as_ref()
    }

    pub fn can_use(&self, source: &CS) -> bool
    where
        R: CommandRequirement<CS>,
    {
        self.requirement.test(source)
    }

    pub fn children(&self) -> indexmap::map::Values<String, NodeId> {
        self.edges.children.values()
    }

    pub async fn list_suggestions<'c, 'i>(ctx: CommandContext<'c, 'i, CS, AT::Value, M>)
    where
        SP: SuggestionProvider<CS>,
        M: RedirectModifier<CS, AT::Value>,
    {
        todo!()
    }

    fn sorted_key(&self) -> &str
    where
        S: AsRef<str>,
    {
        match &self.tagged {
            TaggedCommandNode::Root(_) => "",
            TaggedCommandNode::Literal(l) => l.literal.as_ref(),
            TaggedCommandNode::Argument(a) => a.name.as_ref(),
        }
    }
}

// === NODES ===

#[derive(Clone, Eq, PartialEq)]
pub struct RootCommandNode;

impl fmt::Debug for RootCommandNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<root>")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct LiteralCommandNode<S> {
    literal: S,
    literal_lower_case: Option<String>,
}

impl<S> fmt::Debug for LiteralCommandNode<S>
where
    S: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<literal {}>", self.literal.as_ref())
    }
}

impl<S> LiteralCommandNode<S>
where
    S: AsRef<str>,
{
    pub(crate) fn new(literal: S) -> Self {
        let is_lower = literal.as_ref().chars().all(|c| c.is_ascii_lowercase());
        Self {
            literal_lower_case: (!is_lower).then(|| literal.as_ref().to_ascii_lowercase()),
            literal,
        }
    }

    fn literal(&self) -> &S {
        &self.literal
    }
}

#[derive(Clone)]
pub struct ArgumentCommandNode<AT, SP, S> {
    name: S,
    argument_type: AT,
    custom_suggestions: Option<SP>,
}

impl<AT, SP, S> fmt::Debug for ArgumentCommandNode<AT, SP, S>
where
    S: AsRef<str>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name.as_ref())
    }
}

impl<AT, SP, S> ArgumentCommandNode<AT, SP, S> {
    pub(crate) fn new(name: S, argument_type: AT, custom_suggestions: Option<SP>) -> Self {
        Self {
            name,
            argument_type,
            custom_suggestions,
        }
    }
}
