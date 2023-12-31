use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type, SecondaryMap};

use crate::ast::Span;

pub mod lower;
pub mod display;
pub mod typecheck;

new_key_type! {
    pub struct TypeKey; 
    struct FuncKey;
    struct LiteralKey;
    struct Var;
}

#[derive(Default, Debug)]
pub struct Program<'a> {
    function_names: HashMap<&'a str, FuncKey>,
    functions: SlotMap<FuncKey, Function<'a>>,
    function_decls: SecondaryMap<FuncKey, FunctionDecl<'a>>,
    type_decls: HashMap<&'a str, TypeKey>,
    types: SlotMap<TypeKey, DirectType<'a>>,
    literals: SlotMap<LiteralKey, String>
}

#[derive(Clone, Debug)]
pub enum DirectType<'a> {
    Struct {
        fields: Vec<(&'a str, Type)>
    },
    Union {
        variants: Vec<(&'a str, Type)>
    },
    Enum {
        variants: Vec<(&'a str, i32)>
    },
    Type(Type)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    I32, F32, Bool, U8
}

#[derive(Clone, Debug)]
pub enum Type {
    Direct(TypeKey),
    Primitive(PrimitiveType),
    /// `---`
    Uninit,
    /// `void` type is unit type
    Unit,
    /// type of `break`, `continue` and `return` expressions
    /// can be cast to any type
    Never,
    /// The type is yet unknown, but should get determined during type checking
    /// Using a variable with an undeclared type is an immediate compilation error (and should in theory never happen)
    Undeclared,
    Ptr(Box<Type>),
    Slice(Box<Type>),
    Array { ty: Box<Type>, len: i32 },
    Func {
        ret: Box<Type>,
        params: Vec<Type>
    }
}

#[derive(Debug, Clone)]
struct FunctionDecl<'a> {
    ret: Type,
    params: Vec<Param<'a>>
}

#[derive(Debug, Clone)]
struct Param<'a> {
    outward_name: Option<&'a str>,
    name: &'a str,
    ty: Type
    // value: Option<()>
}

#[derive(Default, Debug)]
struct Function<'a> {
    variables: SlotMap<Var, Variable>,
    body: Block<'a>
}

#[derive(Debug, Default)]
struct Block<'a> {
    stmts: Vec<Statement<'a>>
}

#[derive(Debug)]
struct Variable {
    ty: Type,
    // TODO: metadata: initialized, moved, etc...
}

#[derive(Debug)]
enum Statement<'a> {
    /// Assigns the value of expr to a variable
    Assign(Var, Expr<'a>, Span),
    /// Assigns the value of expr to the location in memory pointed to by a variable
    DerefAssign(Expr<'a>, Expr<'a>, Span),
    /// Assigns the value of an expr into the field of a struct
    FieldAssign {
        object: Expr<'a>,
        field: &'a str,
        value: Expr<'a>,
        span: Span
    },
    Do(Expr<'a>),
    Block(Block<'a>, Span),
    If {
        cond: Expr<'a>,
        block: Block<'a>,
        else_block: Option<Block<'a>>,
        span: Span
    },
    Loop(Block<'a>, Span)
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Var(Var, Span),
    Num(i32, Span),
    Literal(LiteralKey, Span),
    Uninit(Span),
    Unit(Span)
}

#[derive(Debug)]
enum Expr<'a> {
    Value(Value),
    FieldAccess(Value, &'a str, Span),
    PathAccess(TypeKey, &'a str, Span),
    FuncCall(FuncKey, Vec<Value>, Span),
    Return(Option<Value>, Span),
    Break(Span),
    Continue(Span),
    BinOp(Value, BinOp, Value, Span),
    UnaryOp(UnaryOp, Value, Span)
}

#[derive(Debug)]
enum UnaryOp { AddressOf, Deref, Negate, Not }

#[derive(Debug)]
enum BinOp { 
    Add, Sub, Mul, Div, Mod,
    LogicAnd, LogicOr, LogicXor,
    And, Or, Xor,
    Eq, Ne, Gt, Ge, Lt, Le
}

impl Block<'_> {
    /// panics if the block contains no statements
    fn last_expr_span(&self) -> Span {
        self.stmts.last().expect("at least one statement").span()
    }
}

impl Statement<'_> {
    fn span(&self) -> Span {
        match self {
            &Statement::Assign(_, _, span) => span,
            &Statement::DerefAssign(_, _, span) => span,
            &Self::FieldAssign { span, .. } => span,
            Statement::Do(expr) => expr.span(),
            &Statement::Block(_, span) => span,
            &Statement::If { span, .. } => span,
            &Statement::Loop(_, span) => span,
        }
    }
}

impl Value {
    fn expr(self) -> Expr<'static> {
        Expr::Value(self)
    }

    fn span(&self) -> Span {
        match *self {
            Value::Var(_, span) => span,
            Value::Num(_, span) => span,
            Value::Literal(_, span) => span,
            Value::Uninit(span) => span,
            Value::Unit(span) => span,
        }
    }

    fn with_span(self, span: Span) -> Self {
        match self {
            Value::Var(v, _) => Value::Var(v, span),
            Value::Num(n, _) => Value::Num(n, span),
            Value::Literal(l, _) => Value::Literal(l, span),
            Value::Uninit(_) => Value::Uninit(span),
            Value::Unit(_) => Value::Unit(span),
        }
    }
}

impl Expr<'_> {
    fn span(&self) -> Span {
        match self {
            Expr::Value(value) => value.span(),
            &Expr::FieldAccess(_, _, span) => span,
            &Expr::PathAccess(_, _, span) => span,
            &Expr::FuncCall(_, _, span) => span,
            &Expr::Return(_, span) => span,
            &Expr::Break(span) => span,
            &Expr::Continue(span) => span,
            &Expr::BinOp(_, _, _, span) => span,
            &Expr::UnaryOp(_, _, span) => span,
        }
    }
}
