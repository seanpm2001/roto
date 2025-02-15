use std::num::ParseIntError;

use crate::{
    ast::{
        AccessExpr, AccessReceiver, AnonymousRecordValueExpr, ArgExprList,
        AsnLiteral, BooleanLiteral, ComputeExpr, ExtendedCommunityLiteral,
        FieldAccessExpr, HexLiteral, Identifier, IntegerLiteral, IpAddress,
        Ipv4Addr, Ipv6Addr, LargeCommunityLiteral, ListValueExpr,
        LiteralAccessExpr, LiteralExpr, MethodComputeExpr, Prefix,
        PrefixLength, PrefixLengthLiteral, PrefixLengthRange,
        PrefixMatchExpr, PrefixMatchType, StandardCommunityLiteral,
        StringLiteral, TypeIdentifier, TypedRecordValueExpr, ValueExpr,
    },
    parser::ParseError,
};

use super::{
    span::{Spanned, WithSpan},
    token::Token,
    ParseResult, Parser,
};

/// # Parsing value expressions
impl<'source> Parser<'source> {
    /// Parse a value expr
    ///
    /// ```ebnf
    /// ValueExpr ::= '[' ValueExpr* ']
    ///             | Identifier? Record
    ///             | MethodCall
    ///             | Identifier AccessExpr
    ///             | PrefixMatchExpr
    ///             | Literal AccessExpr
    /// ```
    pub(super) fn value_expr(&mut self) -> ParseResult<Spanned<ValueExpr>> {
        if self.peek_is(Token::SquareLeft) {
            let values = self.separated(
                Token::SquareLeft,
                Token::SquareRight,
                Token::Comma,
                Self::value_expr,
            )?;
            let span = values.span;
            return Ok(
                ValueExpr::ListExpr(ListValueExpr { values }).with_span(span)
            );
        }

        if self.peek_is(Token::CurlyLeft) {
            let key_values = self.record()?;
            let span = key_values.span;
            return Ok(ValueExpr::AnonymousRecordExpr(
                AnonymousRecordValueExpr { key_values },
            )
            .with_span(span));
        }

        if let Some(Token::Ident(_)) = self.peek() {
            let id = self.identifier()?;
            if self.peek_is(Token::CurlyLeft) {
                let Identifier { ident: s } = id.inner;
                let type_id = TypeIdentifier { ident: s }.with_span(id.span);
                let key_values = self.record()?;
                let span = id.span.merge(key_values.span);

                return Ok(ValueExpr::TypedRecordExpr(
                    TypedRecordValueExpr {
                        type_id,
                        key_values,
                    }
                    .with_span(span),
                )
                .with_span(span));
            }

            if self.peek_is(Token::RoundLeft) {
                let args = self.arg_expr_list()?;
                let span = id.span.merge(args.args.span);
                return Ok(ValueExpr::RootMethodCallExpr(
                    MethodComputeExpr { ident: id, args },
                )
                .with_span(span));
            }

            let mut span = id.span;

            let receiver = AccessReceiver::Ident(id);
            let access_expr = self.access_expr()?;

            if let Some(last) = access_expr.last() {
                span = span.merge(last.span);
            }

            return Ok(ValueExpr::ComputeExpr(
                ComputeExpr {
                    receiver,
                    access_expr,
                }
                .with_span(span),
            )
            .with_span(span));
        }

        let literal = self.literal()?;

        // If we parsed a prefix, it may be followed by a prefix match
        // If not, it can be an access expression
        if let LiteralExpr::PrefixLiteral(prefix) = &literal.inner {
            if let Some(ty) = self.try_prefix_match_type()? {
                return Ok(ValueExpr::PrefixMatchExpr(PrefixMatchExpr {
                    prefix: prefix.clone(),
                    ty,
                })
                .with_span(literal.span));
            }
        }

        let access_expr = self.access_expr()?;
        let mut span = literal.span;
        if let Some(last) = access_expr.last() {
            span = span.merge(last.span);
        }
        Ok(ValueExpr::LiteralAccessExpr(
            LiteralAccessExpr {
                literal,
                access_expr,
            }
            .with_span(span),
        )
        .with_span(span))
    }

    /// Parse an access expresion
    ///
    /// ```ebnf
    /// AccessExpr ::= ( '.' ( MethodCallExpr | FieldAccessExpr ) )*
    /// ```
    fn access_expr(&mut self) -> ParseResult<Vec<Spanned<AccessExpr>>> {
        let mut access_expr = Vec::new();

        while self.next_is(Token::Period) {
            let ident = self.identifier()?;
            if self.peek_is(Token::RoundLeft) {
                let args = self.arg_expr_list()?;
                let span = ident.span.merge(args.args.span);
                access_expr.push(
                    AccessExpr::MethodComputeExpr(MethodComputeExpr {
                        ident,
                        args,
                    })
                    .with_span(span),
                )
            } else {
                let span = ident.span;

                if let Some(expr) = access_expr.last_mut() {
                    if let AccessExpr::FieldAccessExpr(field_access) =
                        &mut expr.inner
                    {
                        field_access.field_names.push(ident);
                        expr.span = expr.span.merge(span);
                        continue;
                    }
                }

                access_expr.push(
                    AccessExpr::FieldAccessExpr(FieldAccessExpr {
                        field_names: vec![ident],
                    })
                    .with_span(span),
                )
            }
        }

        Ok(access_expr)
    }

    /// Parse any literal, including prefixes, ip addresses and communities
    fn literal(&mut self) -> ParseResult<Spanned<LiteralExpr>> {
        // A prefix length, it requires two tokens
        if let Some(Token::PrefixLength(..)) = self.peek() {
            let prefix_length = self.prefix_length()?;
            let PrefixLength(len) = prefix_length.inner;
            return Ok(LiteralExpr::PrefixLengthLiteral(
                PrefixLengthLiteral(len),
            )
            .with_span(prefix_length.span));
        }

        // If we see an IpAddress, we need to check whether it is followed by a
        // slash and is therefore a prefix instead.
        if matches!(self.peek(), Some(Token::IpV4(_) | Token::IpV6(_))) {
            let addr = self.ip_address()?;
            if let Some(Token::PrefixLength(..)) = self.peek() {
                let len = self.prefix_length()?;
                let span = addr.span.merge(len.span);
                return Ok(LiteralExpr::PrefixLiteral(Prefix { addr, len })
                    .with_span(span));
            } else {
                return Ok(LiteralExpr::IpAddressLiteral(addr.inner)
                    .with_span(addr.span));
            }
        }

        self.simple_literal()
    }

    fn ip_address(&mut self) -> ParseResult<Spanned<IpAddress>> {
        let (token, span) = self.next()?;
        let addr = match token {
            Token::IpV4(s) => IpAddress::Ipv4(Ipv4Addr(
                s.parse::<std::net::Ipv4Addr>().map_err(|e| {
                    ParseError::invalid_literal("Ipv4 addresss", s, e, span)
                })?,
            )),
            Token::IpV6(s) => IpAddress::Ipv6(Ipv6Addr(
                s.parse::<std::net::Ipv6Addr>().map_err(|e| {
                    ParseError::invalid_literal("Ipv6 addresss", s, e, span)
                })?,
            )),
            _ => {
                return Err(ParseError::expected(
                    "an IP address",
                    token,
                    span,
                ))
            }
        };
        Ok(addr.with_span(span))
    }

    /// Parse literals that need no complex parsing, just one token
    fn simple_literal(&mut self) -> ParseResult<Spanned<LiteralExpr>> {
        // TODO: Make proper errors using the spans
        let (token, span) = self.next()?;
        let literal = match token {
            Token::String(s) => {
                // Trim the quotes from the string literal
                let trimmed = &s[1..s.len() - 1];
                LiteralExpr::StringLiteral(StringLiteral(trimmed.into()))
            }
            Token::Integer(s) => LiteralExpr::IntegerLiteral(IntegerLiteral(
                // This parse fails if the literal is too big,
                // it should be handled properly
                s.parse::<i64>().map_err(|e| {
                    ParseError::invalid_literal("integer", token, e, span)
                })?,
            )),
            Token::Hex(s) => LiteralExpr::HexLiteral(HexLiteral(
                u64::from_str_radix(&s[2..], 16).map_err(|e| {
                    ParseError::invalid_literal(
                        "hexadecimal integer",
                        token,
                        e,
                        span,
                    )
                })?,
            )),
            Token::Asn(s) => LiteralExpr::AsnLiteral(AsnLiteral(
                s[2..].parse::<u32>().map_err(|e| {
                    ParseError::invalid_literal("AS number", token, e, span)
                })?,
            )),
            Token::Bool(b) => LiteralExpr::BooleanLiteral(BooleanLiteral(b)),
            Token::Float => {
                unimplemented!("Floating point numbers are not supported yet")
            }
            Token::Community(s) => {
                // We offload the validation of the community to routecore
                // but routecore doesn't do all the hex numbers correctly,
                // so we transform those first.

                // TODO: Change the AST so that it doesn't contain strings, but
                // routecore communities.
                use routecore::bgp::communities::Community;

                let parts = s
                    .split(':')
                    .map(|p| {
                        if let Some(hex) = p.strip_prefix("0x") {
                            Ok(u32::from_str_radix(hex, 16)?.to_string())
                        } else {
                            Ok(p.to_string())
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e: ParseIntError| {
                        ParseError::invalid_literal(
                            "community",
                            s,
                            e,
                            span,
                        )
                    })?;

                let transformed = parts.join(":");

                let c: Community =
                    transformed.parse::<Community>().map_err(|e| {
                        ParseError::invalid_literal(
                            "community",
                            token,
                            e,
                            span,
                        )
                    })?;
                match c {
                    Community::Standard(x) => {
                        LiteralExpr::StandardCommunityLiteral(
                            StandardCommunityLiteral(x),
                        )
                    }
                    Community::Extended(x) => {
                        LiteralExpr::ExtendedCommunityLiteral(
                            ExtendedCommunityLiteral(x),
                        )
                    }
                    Community::Large(x) => {
                        LiteralExpr::LargeCommunityLiteral(
                            LargeCommunityLiteral(x),
                        )
                    }
                    Community::Ipv6Extended(_) => {
                        unimplemented!(
                            "IPv6 extended communities are not supported yet"
                        )
                    }
                }
            }
            t => return Err(ParseError::expected("a literal", t, span)),
        };
        Ok(literal.with_span(span))
    }

    /// Parse an (anonymous) record
    ///
    /// ```ebnf
    /// Record      ::= '{' (RecordField (',' RecordField)* ','? )? '}'
    /// RecordField ::= Identifier ':' ValueExpr
    /// ```
    #[allow(clippy::type_complexity)]
    fn record(
        &mut self,
    ) -> ParseResult<Spanned<Vec<(Spanned<Identifier>, Spanned<ValueExpr>)>>>
    {
        self.separated(
            Token::CurlyLeft,
            Token::CurlyRight,
            Token::Comma,
            |parser| {
                let key = parser.identifier()?;
                parser.take(Token::Colon)?;
                let value = parser.value_expr()?;
                Ok((key, value))
            },
        )
    }

    /// Parse a list of arguments to a method
    ///
    /// ```ebnf
    /// ArgExprList ::= '(' ( ValueExpr (',' ValueExpr)* ','? )? ')'
    /// ```
    pub(super) fn arg_expr_list(&mut self) -> ParseResult<ArgExprList> {
        let args = self.separated(
            Token::RoundLeft,
            Token::RoundRight,
            Token::Comma,
            Self::value_expr,
        )?;

        Ok(ArgExprList { args })
    }

    /// Parse a prefix match type, which can follow a prefix in some contexts
    ///
    /// ```ebnf
    /// PrefixMatchType ::= 'longer'
    ///                   | 'orlonger'
    ///                   | 'prefix-length-range' PrefixLengthRange
    ///                   | 'upto' PrefixLength
    ///                   | 'netmask' IpAddress
    /// ```
    fn try_prefix_match_type(
        &mut self,
    ) -> ParseResult<Option<PrefixMatchType>> {
        let match_type = if self.next_is(Token::Exact) {
            PrefixMatchType::Exact
        } else if self.next_is(Token::Longer) {
            PrefixMatchType::Longer
        } else if self.next_is(Token::OrLonger) {
            PrefixMatchType::OrLonger
        } else if self.next_is(Token::PrefixLengthRange) {
            PrefixMatchType::PrefixLengthRange(
                self.prefix_length_range()?.inner,
            )
        } else if self.next_is(Token::UpTo) {
            PrefixMatchType::UpTo(self.prefix_length()?.inner)
        } else if self.next_is(Token::NetMask) {
            PrefixMatchType::NetMask(self.ip_address()?.inner)
        } else {
            return Ok(None);
        };

        Ok(Some(match_type))
    }

    /// Parse a prefix length range
    ///
    /// ```ebnf
    /// PrefixLengthRange ::= PrefixLength '-' PrefixLength
    /// ```
    fn prefix_length_range(
        &mut self,
    ) -> ParseResult<Spanned<PrefixLengthRange>> {
        let start = self.prefix_length()?;
        self.take(Token::Hyphen)?;
        let end = self.prefix_length()?;
        let span = start.span.merge(end.span);
        Ok(PrefixLengthRange {
            start: start.inner,
            end: end.inner,
        }
        .with_span(span))
    }

    /// Parse a prefix length
    ///
    /// ```ebnf
    /// PrefixLength ::= '/' Integer
    /// ```
    fn prefix_length(&mut self) -> ParseResult<Spanned<PrefixLength>> {
        let (token, span) = self.next()?;
        let Token::PrefixLength(s) = token else {
            return Err(ParseError::invalid_literal(
                "prefix length",
                token,
                "",
                span,
            ));
        };
        let len = s[1..].parse::<u8>().map_err(|e| {
            ParseError::invalid_literal("prefix length", token, e, span)
        })?;
        Ok(PrefixLength(len).with_span(span))
    }
}
