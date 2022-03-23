use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use slotmap::{SecondaryMap, SlotMap};

use crate::{command::Command, context::CommandContext, suggestion::SuggestionProvider, CommandSource};

slotmap::new_key_type! {
    pub struct CommandNodeId;
}
type NodeId = CommandNodeId;

pub struct Tree<'i, S>
where
    S: CommandSource,
{
    strings: HashSet<Rc<str>>,
    nodes: SlotMap<NodeId, CommandNodeComponent<'i, S>>,
    literals: SecondaryMap<NodeId, LiteralCommandNodeComponent>,
    arguments: SecondaryMap<NodeId, ArgumentCommandNodeComponent<S>>,
}

impl<'i, S> Tree<'i, S>
where
    S: CommandSource,
{
    #[inline]
    pub fn add_node(&mut self, node: impl TreeNode<'i, S>) -> NodeId {
        node.add_to_tree(self)
    }
    fn get_shared_str(&mut self, string: &str) -> Rc<str> {
        // TODO: https://github.com/rust-lang/rust/issues/60896
        //Rc::clone(self.strings.get_or_insert_with(string, Rc::new))
        if let Some(s) = self.strings.get(string) {
            s.clone()
        } else {
            let s = Rc::from(string);
            self.strings.insert(Rc::clone(&s));
            s
        }
    }
    /// Deallocates all unused shared strings and returns the amount
    /// of deallocated strings.
    fn collect_garbage(&mut self) -> usize {
        let mut flagged = Vec::with_capacity(self.strings.len());
        for s in &self.strings {
            if Rc::strong_count(s) == 1 {
                flagged.push(Rc::clone(s));
            }
        }
        for s in &flagged {
            self.strings.remove(s);
        }
        flagged.len()
    }
    pub fn add_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), ()> {
        if let Some([parent, child]) = self.nodes.get_disjoint_mut([parent_id, child_id]) {
            let child_name = match child.node_type {
                CommandNodeType::Root => return Err(()),
                CommandNodeType::Argument => {
                    Rc::clone(&unsafe { self.arguments.get_unchecked(child_id) }.name)
                }
                CommandNodeType::Literal => {
                    Rc::clone(&unsafe { self.literals.get_unchecked(child_id) }.literal)
                }
            };
            match parent.children.get(&child_name) {
                Some(&e_child_id) => {
                    // We've found something to merge onto
                    let grandchildren: Vec<_> = child.children.values().cloned().collect();
                    if let Some(command) = child.command {
                        let e_child = self.nodes.get_mut(e_child_id).unwrap();
                        e_child.command = Some(command);
                    }
                    for grandchild_id in grandchildren {
                        self.add_child(e_child_id, grandchild_id).unwrap()
                    }
                }
                None => {
                    parent.children.insert(Rc::clone(&child_name), child_id);
                    match child.node_type {
                        CommandNodeType::Root => unsafe { std::hint::unreachable_unchecked() },
                        CommandNodeType::Argument => {
                            parent.arguments.insert(child_name, child_id);
                        }
                        CommandNodeType::Literal => {
                            parent.literals.insert(child_name, child_id);
                        }
                    }
                }
            }
            return Ok(());
        }
        Err(())
    }
    pub fn find_ambiguities<F>()
    where
        F: FnMut(NodeId, NodeId, NodeId, HashSet<Rc<str>>),
    {
        todo!()
    }
    unsafe fn unchecked_name_of(&mut self, node_id: NodeId, node_type: CommandNodeType) -> Rc<str> {
        match node_type {
            CommandNodeType::Root => self.get_shared_str(""),
            CommandNodeType::Literal => Rc::clone(&self.literals.get_unchecked(node_id).literal),
            CommandNodeType::Argument => Rc::clone(&self.arguments.get_unchecked(node_id).name),
        }
    }
}

pub struct CommandNodeComponent<'i, S>
where
    S: CommandSource,
{
    node_type: CommandNodeType,
    children: HashMap<Rc<str>, NodeId>,
    literals: HashMap<Rc<str>, NodeId>,
    arguments: HashMap<Rc<str>, NodeId>,
    requirement: fn(S) -> bool,
    redirect: Option<NodeId>,
    redirect_modifier: Option<RedirectModifier<'i, S>>,
    forks: bool,
    command: Option<Command<'i, S>>,
}

pub type RedirectModifier<'i, S> = fn(&CommandContext<'i, S>) -> Vec<S>;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandNodeType {
    Root = 0,
    Argument = 1,
    Literal = 2,
}

impl CommandNodeType {
    pub fn is_valid_input(self) -> bool {
        match self {
            Self::Root => false,
            _ => false,
        }
    }
}

pub struct ArgumentCommandNodeComponent<S> {
    name: Rc<str>,
    custom_suggestions: S,
}

pub struct LiteralCommandNodeComponent {
    literal: Rc<str>,
    literal_lower_case: Rc<str>,
}

pub trait TreeNode<'i, S>
where
    S: CommandSource,
{
    fn add_to_tree(self, tree: &mut Tree<'i, S>) -> NodeId;
}

pub struct RootCommandNode;

impl<'i, S> TreeNode<'i, S> for RootCommandNode
where
    S: CommandSource,
{
    fn add_to_tree(self, tree: &mut Tree<'i, S>) -> NodeId {
        tree.nodes.insert(CommandNodeComponent {
            node_type: CommandNodeType::Root,
            children: HashMap::new(),
            literals: HashMap::new(),
            arguments: HashMap::new(),
            requirement: tautology_predicate,
            redirect: None,
            redirect_modifier: Some(|ctx| vec![ctx.source.clone()]),
            forks: false,
            command: None,
        })
    }
}

pub struct ArgumentCommandNode<'a, 'i, 't, 'm, S>
where
    S: CommandSource,
{
    name: &'a str,
    argument_type: ArgumentType,
    command: Option<Command<'i, S>>,
    requirement: fn(S) -> bool,
    redirect: Option<NodeId>,
    modifier: Option<RedirectModifier<'i, S>>,
    forks: bool,
    custom_suggestions: Option<SuggestionProvider<'i, 't, 'm, S>>,
}

pub enum ArgumentType {

}

pub struct LiteralCommandNode<'a, 'i, S>
where
    S: CommandSource,
{
    literal: &'a str,
    command: Option<Command<'i, S>>,
    requirement: fn(S) -> bool,
    redirect: Option<NodeId>,
    modifier: Option<RedirectModifier<'i, S>>,
    forks: bool,
}

/// A predicate that always returns `true` for any argument.
fn tautology_predicate<T>(_: T) -> bool {
    true
}
