use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Formatter},
    hash::Hash,
    rc::Rc,
};

use crate::{
    ast::{CompareOp, ShortString},
    traits::Token,
    types::{
        builtin::BuiltinTypeValue, typedef::TypeDef, typevalue::TypeValue,
    },
};

//------------ Symbols ------------------------------------------------------

// The only symbols we really have are variables & (user-defined) types.

#[derive(Debug)]
pub(crate) struct Symbol {
    name: ShortString,
    kind: SymbolKind,
    ty: TypeDef,
    args: Vec<Symbol>,
    value: Option<TypeValue>,
    token: Option<Token>, // location: Location,
}

impl Symbol {

    // gets the type only from the `ty` field.
    pub fn get_type_and_token(
        &self,
    ) -> Result<(TypeDef, Token), Box<dyn std::error::Error>> {
        let token = self.get_token()?;
        Ok((self.ty.clone(), token))
    }

    pub fn is_initialised(&self) -> bool {
        self.value.is_some()
    }

    pub fn get_builtin_type(
        &self,
    ) -> Result<TypeDef, Box<dyn std::error::Error>> {
        if !matches!(
            self.ty,
            TypeDef::Rib(_)
                | TypeDef::Table(_)
                | TypeDef::List(_)
                | TypeDef::Record(_)
                | TypeDef::None
        ) {
            (&self.ty).try_into().map(|tv: BuiltinTypeValue| tv.into())
        } else if let Some(TypeValue::Builtin(ty)) = &self.value {
            Ok(ty.into())
        } else {
            Err(format!("a Type '{:?}' is not a builtin type", self).into())
        }
    }

    pub fn get_builtin_type_for_leaf_node(
        &self,
    ) -> Result<TypeDef, Box<dyn std::error::Error>> {
        if let Some(TypeValue::Builtin(tv)) = &self.value {
            Ok(tv.into())
        } else {
            Err(format!("b Not a builtin type {:?}", self).into())
        }
    }

    pub fn set_kind(mut self, kind: SymbolKind) -> Symbol {
        self.kind = kind;
        self
    }

    pub fn get_kind(&self) -> SymbolKind {
        self.kind
    }

    pub fn get_token(&self) -> Result<Token, Box<dyn std::error::Error>> {
        self.token
            .clone()
            .ok_or_else(|| format!("No token found for symbol '{:?}'", self).into())
    }

    pub fn get_type(&self) -> TypeDef {
        self.ty.clone()
    }

    pub fn set_token(mut self, token: Token) -> Self {
        self.token = Some(token);
        self
    }

    pub fn get_name(&self) -> ShortString {
        self.name.clone()
    }

    pub fn get_value(&self) -> Option<&TypeValue> {
        self.value.as_ref()
    }

    pub fn get_value_owned(self) -> Option<TypeValue> {
        self.value
    }

    pub fn set_value(&mut self, value: TypeValue) {
        self.value = Some(value);
    }

    pub fn get_args_owned(self) -> Vec<Symbol> {
        self.args
    }

    pub fn get_args(&self) -> &[Symbol] {
        self.args.as_slice()
    }

    pub fn new(
        name: ShortString,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        token: Option<Token>,
    ) -> Self {
        Symbol {
            name,
            kind,
            ty,
            args,
            value: None,
            token,
        }
    }

    pub fn new_with_value(
        name: ShortString,
        kind: SymbolKind,
        value: TypeValue,
        args: Vec<Symbol>,
        token: Token,
    ) -> Self {
        Symbol {
            name,
            kind,
            ty: (&value).into(),
            args,
            value: Some(value),
            token: Some(token),
        }
    }

    // get the return from this symbol if it's a leaf node, otherwise return
    // the type of the nested leaf node (in the 'args' field). Note that this
    // method will have to change if we're going to allow method calls with
    // trailing method calls/field accesses, like `d().e`. Currently the
    // parser won't allow it.
    pub(crate) fn get_return_type(&self) -> TypeDef {
        if self.args.is_empty() {
            self.ty.clone()
        } else if let Some(first_arg) = self.args.first() {
            println!("first arg {:?}", first_arg);
            first_arg.ty.clone()
        }
        else {
            unreachable!()
        }

    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SymbolKind {
    Variable, // A variable defined by the user
    Constant, // A literal value or a module-level variable
    Argument,
    BuiltInType,
    AnonymousType, // type of a sub-record
    NamedType,     // User-defined type of a record
    RxType,        // type of the incoming payload
    TxType,        // type of the outgoing payload
    Rib,
    Table,
    PrefixList,
    DataSourceMethodCall,
    BuiltInTypeMethodCall,
    VariableMethodCall,
    GlobalMethodCall,
    FieldAccess,
    StringLiteral,
    // term symbols
    LogicalExpr,
    BooleanExpr,
    CompareExpr(CompareOp),
    AndExpr,
    OrExpr,
    NotExpr,
    // apply symbols
    MatchAction,
    NegateMatchAction,
    Action,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Scope {
    Global,
    Module(ShortString),
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Module(name) => write!(f, "module '{}'", name),
        }
    }
}

// A per-module symbol table.
#[derive(Debug)]
pub struct SymbolTable {
    scope: Scope,
    // The special symbols that will be filled in at runtime, once per filter
    // run.
    arguments: HashMap<ShortString, Symbol>,
    // The variables and constants that are defined in the module.
    variables: HashMap<ShortString, Symbol>,
    types: HashMap<ShortString, TypeDef>,
    // The evaluated `term` sections that are defined in the module.
    terms: HashMap<ShortString, Vec<Symbol>>,
    // The evaluated `action` sections that are defined in the module.
    actions: HashMap<ShortString, Vec<Symbol>>,
    // All the `filter` clauses in the `apply` section, the tie actions to
    // terms.
    match_actions: HashMap<ShortString, Vec<Symbol>>,
}

// The global symbol table.
pub type GlobalSymbolTable<'a> = Rc<
    RefCell<
        std::collections::HashMap<
            super::symbols::Scope,
            super::symbols::SymbolTable,
        >,
    >,
>;

struct Location {
    name: ShortString,
    module: ShortString,
    line: usize,
}

impl SymbolTable {
    pub(crate) fn new(module: ShortString) -> Self {
        SymbolTable {
            scope: Scope::Module(module),
            arguments: HashMap::new(),
            variables: HashMap::new(),
            terms: HashMap::new(),
            actions: HashMap::new(),
            match_actions: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub(crate) fn move_symbol_into(
        &mut self,
        key: ShortString,
        mut symbol: Symbol,
    ) -> Result<(), Box<dyn std::error::Error>> {

        if self.variables.contains_key(&key) {
            return Err(format!(
                "Symbol {} already defined in scope {}",
                key, self.scope
            )
            .into());
        }

        symbol =
            symbol.set_token(Token::Variable(self.variables.len() as u8));
        self.variables.insert(key, symbol);
        Ok(())
    }

    pub(crate) fn add_variable(
        &mut self,
        key: ShortString,
        name: Option<ShortString>,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        value: Option<TypeValue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = if let Some(name) = name {
            name
        } else {
            key.clone()
        };

        if self.variables.contains_key(&name) {
            return Err(format!(
                "Symbol {} already defined in scope {}",
                name, self.scope
            )
            .into());
        }

        let token_int = self.variables.len() as u8;

        let token = Some(match kind {
            SymbolKind::Rib | SymbolKind::Table | SymbolKind::PrefixList => {
                Token::DataSource(token_int)
            }
            _ => Token::Variable(token_int),
        });

        self.variables.insert(
            key,
            Symbol {
                name,
                kind,
                ty,
                args,
                value,
                token,
            },
        );
        Ok(())
    }

    pub(crate) fn add_argument(
        &mut self,
        key: ShortString,
        name: Option<ShortString>,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        value: Option<TypeValue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = if let Some(name) = name {
            name
        } else {
            key.clone()
        };

        if self.arguments.contains_key(&name) {
            return Err(format!(
                "Symbol {} already defined in scope {}",
                name, self.scope
            )
            .into());
        }

        let token_int = self.arguments.len() as u8;

        let token = Some(Token::Argument(token_int));

        self.arguments.insert(
            key,
            Symbol {
                name,
                kind,
                ty,
                args,
                value,
                token,
            },
        );
        Ok(())
    }

    pub(crate) fn add_logical_formula(
        &mut self,
        key: ShortString,
        name: Option<ShortString>,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        value: Option<TypeValue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = if let Some(name) = name {
            name
        } else {
            key.clone()
        };

        let token_int = self.terms.len() as u8;

        let token = Some(Token::Term(token_int));

        if self.terms.contains_key(&name) {
            return Err(format!(
                "Term {} already defined in scope {}",
                name, self.scope
            )
            .into());
        }

        if let Some(term) = self.terms.get_mut(&key) {
            term.push(Symbol {
                name,
                kind,
                ty,
                args,
                value,
                token,
            });
        } else {
            self.terms.insert(
                key,
                vec![Symbol {
                    name,
                    kind,
                    ty,
                    args,
                    value,
                    token,
                }],
            );
        };

        Ok(())
    }

    pub(crate) fn add_action(
        &mut self,
        key: ShortString,
        name: Option<ShortString>,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        value: Option<TypeValue>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = if let Some(name) = name {
            name
        } else {
            key.clone()
        };

        if self.actions.contains_key(&name) {
            return Err(format!(
                "Action '{}' already defined in scope {}",
                name, self.scope
            )
            .into());
        }

        let token_int = self.actions.len() as u8;

        let token = Some(Token::Action(token_int));

        if let Some(action) = self.actions.get_mut(&key) {
            action.push(Symbol {
                name,
                kind,
                ty,
                args,
                value,
                token,
            });
        } else {
            self.actions.insert(
                key,
                vec![Symbol {
                    name,
                    kind,
                    ty,
                    args,
                    value,
                    token,
                }],
            );
        };
        Ok(())
    }

    pub(crate) fn add_match_action(
        &mut self,
        key: ShortString,
        name: Option<ShortString>,
        kind: SymbolKind,
        ty: TypeDef,
        args: Vec<Symbol>,
        value: Option<TypeValue>,
        // token is the index of the term that corresponds to the `name` 
        // argument. It's a token for a term.
        token: Token
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = if let Some(name) = name {
            name
        } else {
            key.clone()
        };
        
        if self.match_actions.contains_key(&name) {
            return Err(format!(
                "Match Action '{}' already defined in scope {}",
                name, self.scope
            )
            .into());
        }

        if let Some(match_action) = self.match_actions.get_mut(&key) {
            match_action.push(Symbol {
                name,
                kind,
                ty,
                args,
                value,
                token: Some(token),
            });
        } else {
            self.match_actions.insert(
                key,
                vec![Symbol {
                    name,
                    kind,
                    ty,
                    args,
                    value,
                    token: Some(token),
                }],
            );
        };
        Ok(())
    }

    pub fn add_type(&mut self, name: ShortString, ty: TypeDef) {
        self.types.insert(name, ty);
    }

    pub fn get_type(&self, name: &ShortString) -> Option<&TypeDef> {
        self.types.get(name)
    }

    pub(crate) fn get_symbol(&self, name: &ShortString) -> Option<&Symbol> {
        if let Some(symbol) = self.variables.get(name) {
            return Some(symbol);
        }

        if let Some(symbol) = self.arguments.get(name) {
            return Some(symbol);
        }

        if let Some(symbol) = self.terms.get(name) {
            return Some(&symbol[0]);
        }

        if let Some(symbol) = self.actions.get(name) {
            return Some(&symbol[0]);
        }

        if let Some(symbol) = self.match_actions.get(name) {
            return Some(&symbol[0]);
        }

        None
    }
    
    pub(crate) fn get_variable(
        &self,
        name: &ShortString,
    ) -> Result<&Symbol, Box<dyn std::error::Error>> {
        self.variables
            .get(name)
            .or_else(|| self.arguments.get(name))
            .ok_or_else(|| format!("Symbol '{}' not found", name).into())
    }

    pub(crate) fn get_argument(
        &self,
        name: &ShortString,
    ) -> Result<&Symbol, Box<dyn std::error::Error>> {
        self.arguments
            .get(name)
            .ok_or_else(|| format!("Symbol '{}' not found", name).into())
    }

    pub(crate) fn get_term_name(
        &self,
        name: &ShortString,
    ) -> Result<(TypeDef, Token), Box<dyn std::error::Error>> {
        self.terms
            .get(name)
            .ok_or_else(|| format!("Symbol '{}' not found", name).into())
            .map(|terms| {
                let term = terms.first().unwrap();
                (term.ty.clone(), term.token.clone().unwrap())
            })
    }

    pub(crate) fn get_action(
        &self,
        name: &ShortString,
    ) -> Result<&Vec<Symbol>, Box<dyn std::error::Error>> {
        self.actions
            .get(name)
            .ok_or_else(|| format!("Symbol '{}' not found", name).into())
    }

    pub(crate) fn get_data_source(
        &self,
        name: &ShortString,
    ) -> Result<(TypeDef, Token), Box<dyn std::error::Error>> {
        let src: Result<&Symbol, Box<dyn std::error::Error>> = self
            .variables
            .get(name)
            .ok_or_else(|| format!("Symbol '{}' not found", name).into());

        src.map(|r| match r.get_kind() {
            SymbolKind::Rib => r.get_type_and_token(),
            SymbolKind::Table => {
                Ok((TypeDef::Table(Box::new(r.get_type())), r.get_token()?))
            }
            _ => {
                Err(format!("No data source named '{}' found.", name).into())
            }
        })?
    }
}
