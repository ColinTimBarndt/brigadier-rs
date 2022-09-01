use brigadier::builder::builders::literal;
use brigadier::builder::LiteralArgumentBuilder;
use brigadier::tree::TreeGraph;
use brigadier::{build_literal, ArgumentType, NoRedirect, ParsedValue, Unrestricted};

type Tree = TreeGraph<(), Type, (), Unrestricted, NoRedirect, String, usize>;

#[derive(Eq, PartialEq)]
enum Value {
    Int(i32),
    String(String),
}

impl ParsedValue for Value {}

enum Type {
    Int,
    String,
}

impl ArgumentType for Type {
    type Value = Value;
}

fn main() {
    let mut tree: Tree = TreeGraph::new();
    let mut builder = LiteralArgumentBuilder::new(&mut tree, "test".into());

    fn force_hr<F, A, R>(f: F) -> F
    where
        F: for<'b> FnOnce(&'b mut A) -> R,
    {
        f
    }

    (*builder).then_build(
        literal("foo".into()),
        force_hr(|b: &mut _| {
            //b.then_build(literal("baz".into()), |b: &mut _| {
            //    //
            //});
        }),
    );

    //build_literal!(builder, "bar".into(), b => {
    //    //
    //});
}
