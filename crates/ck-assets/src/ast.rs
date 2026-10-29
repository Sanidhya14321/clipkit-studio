//! Static AST extraction for TS, TSX, JS and JSX (spec 7.1 `ast.json`, 7.3 skeleton and slice).
//! Parsing only: no code is evaluated. Top-level statements give the imports, exports, typed
//! props, the Remotion animation primitives in use and an outline of symbols with line ranges
//! and doc comments, so the agent can read a skeleton or one symbol instead of the source.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Declaration, ExportDefaultDeclarationKind, ImportDeclarationSpecifier, Statement, TSSignature,
    TSType,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Remotion names that tell how an asset animates.
pub const PRIMITIVES: &[&str] = &[
    "useCurrentFrame",
    "useVideoConfig",
    "interpolate",
    "interpolateColors",
    "spring",
    "Easing",
    "Sequence",
    "Series",
    "AbsoluteFill",
    "Img",
    "Audio",
    "Video",
    "OffthreadVideo",
    "staticFile",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AstMap {
    pub file: String,
    pub imports: Vec<Import>,
    pub exports: Vec<String>,
    pub props: Vec<PropsType>,
    /// Remotion primitives imported and used.
    pub primitives: Vec<String>,
    pub outline: Vec<Symbol>,
    /// Recoverable syntax errors (the map may be partial).
    #[serde(default)]
    pub errors: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Import {
    pub from: String,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropsType {
    pub name: String,
    pub fields: Vec<Prop>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub name: String,
    /// The type as written.
    #[serde(rename = "type")]
    pub ty: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub name: String,
    /// function, const, class, interface, type or default.
    pub kind: String,
    pub exported: bool,
    /// 1-based, inclusive.
    pub start_line: u32,
    pub end_line: u32,
    /// The leading `/** ... */` comment, without its markers.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub doc: String,
}

fn text(src: &str, span: Span) -> &str {
    src.get(span.start as usize..span.end as usize)
        .unwrap_or("")
}

fn line(src: &str, offset: u32) -> u32 {
    src.get(..offset as usize)
        .map_or(0, |s| s.bytes().filter(|b| *b == b'\n').count()) as u32
        + 1
}

/// The `/** ... */` comment right before `start`, if any.
fn doc_before(src: &str, start: u32) -> String {
    let before = src.get(..start as usize).unwrap_or("");
    let trimmed = before.trim_end();
    let Some(body) = trimmed.strip_suffix("*/") else {
        return String::new();
    };
    let Some(open) = body.rfind("/**") else {
        return String::new();
    };
    body[open + 3..]
        .lines()
        .map(|l| l.trim().trim_start_matches('*').trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn members(src: &str, sigs: &[TSSignature<'_>]) -> Vec<Prop> {
    sigs.iter()
        .filter_map(|s| match s {
            TSSignature::TSPropertySignature(p) => Some(Prop {
                name: text(src, p.key.span()).trim_matches(['"', '\'']).to_owned(),
                ty: p
                    .type_annotation
                    .as_ref()
                    .map(|t| text(src, t.type_annotation.span()).to_owned())
                    .unwrap_or_default(),
                optional: p.optional,
            }),
            _ => None,
        })
        .collect()
}

struct Out<'s> {
    src: &'s str,
    map: AstMap,
}

impl Out<'_> {
    fn symbol(&mut self, name: &str, kind: &str, exported: bool, span: Span, outer: Span) {
        self.map.outline.push(Symbol {
            name: name.to_owned(),
            kind: kind.to_owned(),
            exported,
            start_line: line(self.src, outer.start),
            end_line: line(self.src, outer.end),
            doc: doc_before(self.src, outer.start),
        });
        let _ = span;
        if exported {
            self.map.exports.push(name.to_owned());
        }
    }

    fn declaration(&mut self, d: &Declaration<'_>, exported: bool, outer: Span) {
        let src = self.src;
        match d {
            Declaration::VariableDeclaration(v) => {
                for decl in &v.declarations {
                    if let Some(name) = decl.id.get_identifier_name() {
                        self.symbol(&name, "const", exported, decl.span, outer);
                    }
                }
            }
            Declaration::FunctionDeclaration(f) => {
                if let Some(id) = &f.id {
                    self.symbol(&id.name, "function", exported, f.span, outer);
                }
            }
            Declaration::ClassDeclaration(c) => {
                if let Some(id) = &c.id {
                    self.symbol(&id.name, "class", exported, c.span, outer);
                }
            }
            Declaration::TSInterfaceDeclaration(i) => {
                self.symbol(&i.id.name, "interface", exported, i.span, outer);
                if i.id.name.ends_with("Props") {
                    self.map.props.push(PropsType {
                        name: i.id.name.to_string(),
                        fields: members(src, &i.body.body),
                    });
                }
            }
            Declaration::TSTypeAliasDeclaration(t) => {
                self.symbol(&t.id.name, "type", exported, t.span, outer);
                if t.id.name.ends_with("Props")
                    && let TSType::TSTypeLiteral(lit) = &t.type_annotation
                {
                    self.map.props.push(PropsType {
                        name: t.id.name.to_string(),
                        fields: members(src, &lit.members),
                    });
                }
            }
            _ => {}
        }
    }
}

/// Parses `src` (file name `file` picks TS, TSX, JS or JSX) into an [`AstMap`].
pub fn extract(file: &str, src: &str) -> AstMap {
    let alloc = Allocator::default();
    let ty = SourceType::from_path(file).unwrap_or_else(|_| SourceType::tsx());
    let parsed = Parser::new(&alloc, src, ty).parse();
    let mut out = Out {
        src,
        map: AstMap {
            file: file.to_owned(),
            imports: Vec::new(),
            exports: Vec::new(),
            props: Vec::new(),
            primitives: Vec::new(),
            outline: Vec::new(),
            errors: parsed.diagnostics.len(),
        },
    };
    for stmt in &parsed.program.body {
        match stmt {
            Statement::ImportDeclaration(i) => {
                let names = i
                    .specifiers
                    .iter()
                    .flatten()
                    .map(|s| match s {
                        ImportDeclarationSpecifier::ImportSpecifier(s) => s.local.name.to_string(),
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                            s.local.name.to_string()
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                            format!("* as {}", s.local.name)
                        }
                    })
                    .collect();
                out.map.imports.push(Import {
                    from: i.source.value.to_string(),
                    names,
                });
            }
            Statement::ExportDeclaration(e) => out.declaration(&e.declaration, true, e.span),
            Statement::ExportNamedDeclaration(e) => {
                for s in &e.specifiers {
                    out.map
                        .exports
                        .push(text(src, s.exported.span()).to_owned());
                }
            }
            Statement::ExportDefaultDeclaration(e) => {
                let name = match &e.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                        f.id.as_ref().map(|i| i.name.to_string())
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(c) => {
                        c.id.as_ref().map(|i| i.name.to_string())
                    }
                    _ => None,
                }
                .unwrap_or_else(|| "default".to_owned());
                out.symbol(&name, "default", true, e.span, e.span);
            }
            Statement::VariableDeclaration(v) => {
                for decl in &v.declarations {
                    if let Some(name) = decl.id.get_identifier_name() {
                        out.symbol(&name, "const", false, decl.span, v.span);
                    }
                }
            }
            Statement::FunctionDeclaration(f) => {
                if let Some(id) = &f.id {
                    out.symbol(&id.name, "function", false, f.span, f.span);
                }
            }
            Statement::TSInterfaceDeclaration(_) | Statement::TSTypeAliasDeclaration(_) => {
                if let Some(d) = stmt.as_declaration() {
                    out.declaration(d, false, stmt.span());
                }
            }
            _ => {}
        }
    }
    let remotion: Vec<&str> = out
        .map
        .imports
        .iter()
        .filter(|i| i.from == "remotion" || i.from.starts_with("@remotion/"))
        .flat_map(|i| i.names.iter().map(String::as_str))
        .collect();
    out.map.primitives = PRIMITIVES
        .iter()
        .filter(|p| remotion.contains(p))
        .map(|p| (*p).to_owned())
        .collect();
    out.map
}

/// The lines `start..=end` (1-based) of `src`.
pub fn slice(src: &str, start: u32, end: u32) -> String {
    src.lines()
        .skip(start.saturating_sub(1) as usize)
        .take((end + 1).saturating_sub(start) as usize)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests;
