use crate::parser::meta::Meta;

#[derive(Clone, Debug)]
pub struct SyntaxTree {
    pub expressions: Vec<Declaration>,
}

#[derive(Clone, Debug)]
pub enum Declaration {
    FilterMap(Box<FilterMap>),
    Rib(Rib),
    Table(Table),
    OutputStream(OutputStream),
    Record(RecordTypeDeclaration),
    Function(FunctionDeclaration),
}

#[derive(Clone, Debug)]
pub struct Params(pub Vec<(Meta<Identifier>, Meta<Identifier>)>);

/// The value of a typed record
#[derive(Clone, Debug)]
pub struct RecordTypeDeclaration {
    pub ident: Meta<Identifier>,
    pub record_type: RecordType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterType {
    FilterMap,
    Filter,
}

#[derive(Clone, Debug)]
pub struct FilterMap {
    pub filter_type: FilterType,
    pub ident: Meta<Identifier>,
    pub params: Meta<Params>,
    pub body: FilterMapBody,
}

#[derive(Clone, Debug)]
pub struct FilterMapBody {
    pub define: Vec<(Meta<Identifier>, Meta<Expr>)>,
    pub apply: Meta<Block>,
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub ident: Meta<Identifier>,
    pub params: Meta<Params>,
    pub ret: Option<Meta<Identifier>>,
    pub body: Meta<Block>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub exprs: Vec<Meta<Expr>>,
    pub last: Option<Box<Meta<Expr>>>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Return(ReturnKind, Option<Box<Meta<Expr>>>),
    /// a literal, or a chain of field accesses and/or methods on a literal,
    /// e.g. `10.0.0.0/8.covers(..)`
    Literal(Meta<Literal>),
    Match(Box<Meta<Match>>),
    /// a JunOS style prefix match expression, e.g. `0.0.0.0/0
    /// prefix-length-range /12-/16`
    PrefixMatch(PrefixMatchExpr),
    FunctionCall(Meta<Identifier>, Meta<Vec<Meta<Expr>>>),
    MethodCall(Box<Meta<Expr>>, Meta<Identifier>, Meta<Vec<Meta<Expr>>>),
    Access(Box<Meta<Expr>>, Meta<Identifier>),
    Var(Meta<Identifier>),
    /// a record that doesn't have a type mentioned in the assignment of it,
    /// e.g `{ value_1: 100, value_2: "bla" }`. This can also be a sub-record
    /// of a record that does have an explicit type.
    Record(Meta<Record>),
    /// an expression of a record that does have a type, e.g. `MyType {
    /// value_1: 100, value_2: "bla" }`, where MyType is a user-defined Record
    /// Type.
    TypedRecord(Meta<Identifier>, Meta<Record>),
    /// An expression that yields a list of values, e.g. `[100, 200, 300]`
    List(Vec<Meta<Expr>>),
    Not(Box<Meta<Expr>>),
    BinOp(Box<Meta<Expr>>, BinOp, Box<Meta<Expr>>),
    IfElse(Box<Meta<Expr>>, Meta<Block>, Option<Meta<Block>>),
}

#[derive(Clone, Debug)]
pub enum ReturnKind {
    Return,
    Accept,
    Reject,
}

impl ReturnKind {
    pub fn str(&self) -> &'static str {
        match self {
            ReturnKind::Return => "return",
            ReturnKind::Accept => "accept",
            ReturnKind::Reject => "reject",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    pub fields: Vec<(Meta<Identifier>, Meta<Expr>)>,
}

#[derive(Clone, Debug)]
pub struct Match {
    pub expr: Meta<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub variant_id: Meta<Identifier>,
    pub data_field: Option<Meta<Identifier>>,
    pub guard: Option<Meta<Expr>>,
    pub body: Meta<Block>,
}

#[derive(Clone, Debug)]
pub struct Rib {
    pub ident: Meta<Identifier>,
    pub contain_ty: Meta<Identifier>,
    pub body: RibBody,
}

#[derive(Clone, Debug)]
pub struct RibBody {
    pub key_values: Meta<Vec<(Meta<Identifier>, RibFieldType)>>,
}

#[derive(Clone, Debug)]
pub enum RibFieldType {
    Identifier(Meta<Identifier>),
    Record(Meta<RecordType>),
    List(Meta<Box<RibFieldType>>),
}

#[derive(Clone, Debug)]
pub struct Table {
    pub ident: Meta<Identifier>,
    pub contain_ty: Meta<Identifier>,
    pub body: RibBody,
}

#[derive(Clone, Debug)]
pub struct OutputStream {
    pub ident: Meta<Identifier>,
    pub contain_ty: Meta<Identifier>,
    pub body: RibBody,
}

/// An identifier is the name of variables or other things.
///
/// It is a word composed of a leading alphabetic Unicode character, followed
/// by alphanumeric Unicode characters or underscore or hyphen.
#[derive(Clone, Debug, Ord, PartialOrd, Hash)]
pub struct Identifier(pub String);

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::borrow::Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Identifier {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_ref()
    }
}

impl Eq for Identifier {}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The user-defined type of a record. It's very similar to a RibBody (in EBNF
/// it's the same), but it simplifies creating the SymbolTable, because they're
/// semantically different.
#[derive(Clone, Debug)]
pub struct RecordType {
    pub key_values: Meta<Vec<(Meta<Identifier>, RibFieldType)>>,
}

#[derive(Clone, Debug)]
pub enum Literal {
    String(String),
    Prefix(Prefix),
    PrefixLength(u8),
    Asn(u32),
    IpAddress(IpAddress),
    ExtendedCommunity(routecore::bgp::communities::ExtendedCommunity),
    StandardCommunity(routecore::bgp::communities::StandardCommunity),
    LargeCommunity(routecore::bgp::communities::LargeCommunity),
    Integer(i64),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub enum BinOp {
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    NotIn,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::And => "&&",
                Self::Or => "||",
                Self::Eq => "==",
                Self::Ne => "!=",
                Self::Lt => "<=",
                Self::Le => "<",
                Self::Gt => ">=",
                Self::Ge => "<",
                Self::In => "in",
                Self::NotIn => "not in",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub enum PrefixMatchType {
    Exact,
    Longer,
    OrLonger,
    PrefixLengthRange(PrefixLengthRange),
    UpTo(u8),
    NetMask(IpAddress),
}

#[derive(Clone, Debug)]
pub struct PrefixMatchExpr {
    pub prefix: Prefix,
    pub ty: PrefixMatchType,
}

#[derive(Clone, Debug)]
pub enum IpAddress {
    Ipv4(std::net::Ipv4Addr),
    Ipv6(std::net::Ipv6Addr),
}

#[derive(Clone, Debug)]
pub struct Prefix {
    pub addr: Meta<IpAddress>,
    pub len: Meta<u8>,
}

#[derive(Clone, Debug)]
pub struct PrefixLengthRange {
    pub start: u8,
    pub end: u8,
}
