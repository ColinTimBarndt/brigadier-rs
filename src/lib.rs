#![feature(generic_associated_types)]

extern crate core;

use std::iter;

use crate::context::CommandContext;
use crate::errors::CommandSyntaxError;

pub mod builder;
pub mod context;
pub mod errors;
pub mod tree;

pub trait Command<CS, PV, M> {
    type Result;

    fn run(&self, ctx: CommandContext<CS, PV, M>) -> Result<Self::Result, CommandSyntaxError>
    where
        PV: ParsedValue,
        M: RedirectModifier<CS, PV>;
}

pub trait ParsedValue: PartialEq {}

pub trait ArgumentType {
    type Value: ParsedValue;
}

pub trait SuggestionProvider<CS> {
    // TODO
}

pub trait RedirectModifier<CS, PV>: Sized
where
    PV: ParsedValue,
{
    type Targets: Iterator<Item = CS>;

    fn apply(
        &self,
        ctx: &CommandContext<CS, PV, Self>,
    ) -> Result<Self::Targets, CommandSyntaxError>;
}

pub trait SingleRedirectModifier<CS, PV>: Sized
where
    PV: ParsedValue,
{
    fn apply(&self, ctx: &CommandContext<CS, PV, Self>) -> Result<CS, CommandSyntaxError>;
}

impl<T, CS, PV> RedirectModifier<CS, PV> for T
where
    PV: ParsedValue,
    T: SingleRedirectModifier<CS, PV>,
{
    type Targets = iter::Once<CS>;

    fn apply(&self, ctx: &CommandContext<CS, PV, T>) -> Result<Self::Targets, CommandSyntaxError> {
        SingleRedirectModifier::apply(self, ctx).map(iter::once)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NoRedirect {}

impl<CS, PV> SingleRedirectModifier<CS, PV> for NoRedirect
where
    PV: ParsedValue,
{
    fn apply(&self, _ctx: &CommandContext<CS, PV, Self>) -> Result<CS, CommandSyntaxError> {
        unreachable!()
    }
}

pub trait CommandRequirement<CS> {
    fn always() -> Self;
    fn test(&self, source: &CS) -> bool;
}

#[derive(Debug, Copy, Clone)]
pub struct Unrestricted;

impl<CS> CommandRequirement<CS> for Unrestricted {
    fn always() -> Self {
        Self
    }

    fn test(&self, _source: &CS) -> bool {
        true
    }
}

impl<CS, F> CommandRequirement<CS> for Option<F>
where
    F: Fn(&CS) -> bool,
{
    fn always() -> Self {
        None
    }

    fn test(&self, source: &CS) -> bool {
        match self {
            None => true,
            Some(f) => f(source),
        }
    }
}
