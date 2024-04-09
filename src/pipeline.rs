use std::collections::HashMap;

use log::info;

use crate::{
    ast,
    lower::{
        self, eval,
        ir::{self, SafeValue},
    },
    parser::{
        meta::{MetaId, Span, Spans},
        ParseError, Parser,
    },
    typechecker::{
        error::{Level, TypeError},
        types::Type,
    },
};

#[derive(Clone, Debug)]
pub struct SourceFile {
    name: String,
    contents: String,
}

#[derive(Debug)]
enum RotoError {
    Read(String, std::io::Error),
    Parse(ParseError),
    Type(TypeError),
}

#[derive(Debug)]
pub struct RotoReport {
    files: Vec<SourceFile>,
    errors: Vec<RotoError>,
    spans: Spans,
}

/// Compiler Stages

pub struct LoadedFiles {
    files: Vec<SourceFile>,
}

pub struct Parsed {
    files: Vec<SourceFile>,
    trees: Vec<ast::SyntaxTree>,
    spans: Spans,
}

pub struct TypeChecked {
    trees: Vec<ast::SyntaxTree>,
    expr_types: Vec<HashMap<MetaId, Type>>,
    fully_qualified_names: Vec<HashMap<MetaId, String>>,
}
pub struct Lowered {
    ir: ir::Program<ir::Var, SafeValue>,
}

pub struct Evaluated {
    value: SafeValue,
}

impl std::fmt::Display for RotoReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ariadne::{Color, Label, Report, ReportKind};

        let mut file_cache = ariadne::sources(
            self.files
                .iter()
                .map(|s| (s.name.clone(), s.contents.clone())),
        );

        for error in &self.errors {
            match error {
                RotoError::Read(name, io) => {
                    write!(f, "could not open `{name}`: {io}")?;
                }
                RotoError::Parse(error) => {
                    let label_message = error.kind.label();
                    let label = Label::new((
                        self.filename(error.location),
                        error.location.start..error.location.end,
                    ))
                    .with_message(label_message)
                    .with_color(Color::Red);

                    let file = self.filename(error.location);

                    let report = Report::build(
                        ReportKind::Error,
                        file,
                        error.location.start,
                    )
                    .with_message(format!("Parse error: {}", error))
                    .with_label(label)
                    .finish();

                    let mut v = Vec::new();
                    report.write(&mut file_cache, &mut v).unwrap();
                    let s = String::from_utf8_lossy(&v);
                    write!(f, "{s}")?;
                }
                RotoError::Type(error) => {
                    let labels = error.labels.iter().map(|l| {
                        let s = self.spans.get(l.id);
                        Label::new((self.filename(s), s.start..s.end))
                            .with_message(&l.message)
                            .with_color(match l.level {
                                Level::Error => Color::Red,
                                Level::Info => Color::Blue,
                            })
                    });

                    let file = self.filename(self.spans.get(error.location));

                    let report = Report::build(
                        ReportKind::Error,
                        file,
                        self.spans.get(error.location).start,
                    )
                    .with_message(format!(
                        "Type error: {}",
                        &error.description
                    ))
                    .with_labels(labels)
                    .finish();

                    let mut v = Vec::new();
                    report.write(&mut file_cache, &mut v).unwrap();
                    let s = String::from_utf8_lossy(&v);
                    write!(f, "{s}")?;
                }
            }
        }

        Ok(())
    }
}

impl RotoReport {
    fn filename(&self, s: Span) -> String {
        self.files[s.file].name.clone()
    }
}

impl std::error::Error for RotoReport {}

pub fn run(
    files: impl IntoIterator<Item = String>,
    rx: SafeValue,
) -> Result<Evaluated, RotoReport> {
    let lowered = read_files(files)?.parse()?.typecheck()?.lower();
    info!("Generated code:\n{}", lowered.ir);
    Ok(lowered.eval(rx))
}

pub fn test_file(source: &str) -> LoadedFiles {
    LoadedFiles {
        files: vec![SourceFile {
            name: "test".into(),
            contents: source.into(),
        }],
    }
}

fn read_files(
    files: impl IntoIterator<Item = String>,
) -> Result<LoadedFiles, RotoReport> {
    let results: Vec<_> = files
        .into_iter()
        .map(|f| (f.to_string(), std::fs::read_to_string(f)))
        .collect();

    let mut files = Vec::new();
    let mut errors = Vec::new();
    for (name, result) in results {
        match result {
            Ok(contents) => files.push(SourceFile { name, contents }),
            Err(err) => {
                errors.push(RotoError::Read(name.clone(), err));
                files.push(SourceFile {
                    name,
                    contents: String::new(),
                });
            }
        };
    }

    if errors.is_empty() {
        Ok(LoadedFiles { files })
    } else {
        Err(RotoReport {
            files,
            errors,
            spans: Spans::default(),
        })
    }
}

impl LoadedFiles {
    pub fn parse(self) -> Result<Parsed, RotoReport> {
        let mut spans = Spans::default();

        let results: Vec<_> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, f)| Parser::parse(i, &mut spans, &f.contents))
            .collect();

        let mut trees = Vec::new();
        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok(tree) => trees.push(tree),
                Err(err) => errors.push(RotoError::Parse(err)),
            };
        }

        if errors.is_empty() {
            Ok(Parsed {
                trees,
                spans,
                files: self.files,
            })
        } else {
            Err(RotoReport {
                files: self.files.to_vec(),
                errors,
                spans,
            })
        }
    }
}

impl Parsed {
    pub fn typecheck(self) -> Result<TypeChecked, RotoReport> {
        let Parsed {
            files,
            trees,
            spans,
        } = self;

        let results: Vec<_> = trees
            .iter()
            .map(|f| crate::typechecker::typecheck(f))
            .collect();

        let mut expr_types = Vec::new();
        let mut fully_qualified_types = Vec::new();
        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok((type_map, name_map)) => {
                    expr_types.push(type_map);
                    fully_qualified_types.push(name_map);
                }
                Err(err) => errors.push(RotoError::Type(err)),
            }
        }

        if errors.is_empty() {
            Ok(TypeChecked {
                trees,
                expr_types,
                fully_qualified_names: fully_qualified_types,
            })
        } else {
            Err(RotoReport {
                files: files.to_vec(),
                errors,
                spans,
            })
        }
    }
}

impl TypeChecked {
    pub fn lower(self) -> Lowered {
        let TypeChecked {
            trees,
            expr_types,
            fully_qualified_names,
        } = self;
        let ir = lower::lower(
            &trees[0],
            expr_types[0].clone(),
            fully_qualified_names[0].clone(),
        );
        Lowered { ir }
    }
}

impl Lowered {
    pub fn eval(self, rx: SafeValue) -> Evaluated {
        Evaluated {
            value: eval::eval(&self.ir, "main", rx),
        }
    }
}

impl Evaluated {
    pub fn to_value(self) -> SafeValue {
        self.value
    }
}
