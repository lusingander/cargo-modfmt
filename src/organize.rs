mod imports;
mod layout;
mod ordering;

use std::{cmp::Ordering, collections::HashMap, error::Error, fmt, ops::Range, str::FromStr};

use ra_ap_syntax::ast::{HasAttrs, HasModuleItem, HasName, HasVisibility};
use ra_ap_syntax::{AstNode, Edition, SyntaxKind, SyntaxToken, TextRange, ast};

use crate::organize::{
    imports::{ImportBinding, ImportKind, ImportTree, identifier_name},
    layout::{IMPORT_WIDTH, column_after},
    ordering::compare_trees,
};

#[derive(Clone, Copy, Debug)]
pub struct FormatOptions {
    pub edition: SourceEdition,
    pub style_edition: StyleEdition,
}

#[derive(Clone, Copy, Debug)]
pub enum SourceEdition {
    E2015,
    E2018,
    E2021,
    E2024,
}

impl FromStr for SourceEdition {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "2015" => Ok(Self::E2015),
            "2018" => Ok(Self::E2018),
            "2021" => Ok(Self::E2021),
            "2024" => Ok(Self::E2024),
            _ => Err(format!(
                "invalid Rust edition '{value}' (expected 2015, 2018, 2021, or 2024)"
            )),
        }
    }
}

impl SourceEdition {
    fn parser_edition(self) -> Edition {
        match self {
            Self::E2015 => Edition::Edition2015,
            Self::E2018 => Edition::Edition2018,
            Self::E2021 => Edition::Edition2021,
            Self::E2024 => Edition::Edition2024,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StyleEdition {
    E2015,
    E2018,
    E2021,
    E2024,
}

impl From<SourceEdition> for StyleEdition {
    fn from(edition: SourceEdition) -> Self {
        match edition {
            SourceEdition::E2015 => Self::E2015,
            SourceEdition::E2018 => Self::E2018,
            SourceEdition::E2021 => Self::E2021,
            SourceEdition::E2024 => Self::E2024,
        }
    }
}

impl FromStr for StyleEdition {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<SourceEdition>().map(Self::from).map_err(|_| {
            format!("invalid Rust style edition '{value}' (expected 2015, 2018, 2021, or 2024)")
        })
    }
}

#[derive(Debug)]
pub struct OrganizeError {
    message: String,
    line: usize,
    column: usize,
}

impl fmt::Display for OrganizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "syntax error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl Error for OrganizeError {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DeclarationGroup {
    PrivateModule,
    PublicModule,
    StandardUse,
    ExternalUse,
    LocalUse,
}

struct Declaration {
    group: DeclarationGroup,
    range: Range<usize>,
    visibility: String,
    kind: DeclarationKind,
}

enum DeclarationKind {
    Module(String),
    Use(ImportTree),
}

#[derive(PartialEq, Eq, Hash)]
struct MergeKey {
    root: Vec<String>,
    absolute: bool,
    visibility: String,
}

impl ImportTree {
    fn merge_key(&self, visibility: &str) -> MergeKey {
        let root_length = match self.path[0].as_str() {
            "self" | "super" => {
                1 + self
                    .path
                    .iter()
                    .skip(1)
                    .take_while(|part| part.as_str() == "super")
                    .count()
            }
            _ => 1,
        };
        MergeKey {
            root: self.path[..root_length].to_vec(),
            absolute: self.absolute,
            visibility: visibility.to_owned(),
        }
    }
}

struct DeclarationRun<'a> {
    range: Range<usize>,
    declarations: Vec<Declaration>,
    newline: &'static str,
    indent: &'a str,
}

impl DeclarationRun<'_> {
    fn write(&self, output: &mut String) {
        let mut previous_group = None;
        for declaration in &self.declarations {
            if let Some(group) = previous_group {
                output.push_str(self.newline);
                if group != declaration.group {
                    output.push_str(self.newline);
                }
                output.push_str(self.indent);
            }
            output.push_str(&declaration.visibility);
            if !declaration.visibility.is_empty() {
                output.push(' ');
            }
            match &declaration.kind {
                DeclarationKind::Use(tree) => {
                    output.push_str("use ");
                    let line = output
                        .strip_prefix('\u{feff}')
                        .unwrap_or(output)
                        .rsplit('\n')
                        .next()
                        .unwrap_or("");
                    let column = column_after(0, line);
                    tree.write_layout(
                        output,
                        IMPORT_WIDTH.saturating_sub(column + 1),
                        self.indent,
                        self.newline,
                    );
                }
                DeclarationKind::Module(name) => {
                    output.push_str("mod ");
                    output.push_str(name);
                }
            }
            output.push(';');
            previous_group = Some(declaration.group);
        }
    }
}

struct Organizer<'a> {
    style: StyleEdition,
    source: &'a str,
    comments: Vec<TextRange>,
    file_endings: LineEndings,
    runs: Vec<DeclarationRun<'a>>,
}

#[derive(Clone, Copy)]
enum LineEndings {
    None,
    Lf,
    CrLf,
    Mixed,
}

pub fn organize(source: &str, options: FormatOptions) -> Result<String, OrganizeError> {
    let body = source.strip_prefix('\u{feff}').unwrap_or(source);
    let prefix = &source[..source.len() - body.len()];
    let parse = ast::SourceFile::parse(body, options.edition.parser_edition());
    if let Some(error) = parse.errors().first() {
        let before = &body[..usize::from(error.range().start())];
        return Err(OrganizeError {
            message: error.to_string(),
            line: before.bytes().filter(|&byte| byte == b'\n').count() + 1,
            column: before.rsplit('\n').next().unwrap_or("").chars().count() + 1,
        });
    }

    let file = parse.tree();
    let items: Vec<_> = file.items().collect();
    let comments: Vec<_> = file
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| token.kind() == SyntaxKind::COMMENT && !inside_attribute(token))
        .map(|token| token.text_range())
        .collect();
    let mut organizer = Organizer {
        style: options.style_edition,
        source: body,
        comments,
        file_endings: line_endings(body),
        runs: Vec::new(),
    };
    organizer.collect_scope(
        &items,
        0..body.len(),
        !file.attrs_with_doc().all(allows_inner_header),
    );
    organizer.runs.sort_by_key(|run| run.range.start);
    let mut output = String::with_capacity(source.len());
    output.push_str(prefix);
    let mut cursor = 0;
    for run in organizer.runs {
        output.push_str(&body[cursor..run.range.start]);
        run.write(&mut output);
        cursor = run.range.end;
    }
    output.push_str(&body[cursor..]);
    Ok(output)
}

impl Organizer<'_> {
    fn collect_scope(
        &mut self,
        items: &[ast::Item],
        scope_range: Range<usize>,
        has_unsupported_header: bool,
    ) {
        let mut start = 0;
        while start < items.len() {
            if !is_declaration(&items[start]) {
                if let ast::Item::Module(module) = &items[start] {
                    self.collect_module(module);
                }
                start += 1;
                continue;
            }
            let mut end = start + 1;
            while end < items.len() && is_declaration(&items[end]) {
                end += 1;
            }
            if !(start == 0 && has_unsupported_header) {
                let guard_start = if start == 0 {
                    Some(scope_range.start)
                } else {
                    item_content_range(&items[start - 1]).map(|range| usize::from(range.end()))
                };
                let guard_end = if end == items.len() {
                    Some(scope_range.end)
                } else {
                    item_content_range(&items[end]).map(|range| usize::from(range.start()))
                };
                if let (Some(guard_start), Some(guard_end)) = (guard_start, guard_end) {
                    let first_comment = self
                        .comments
                        .partition_point(|range| usize::from(range.end()) <= guard_start);
                    let has_comments = self
                        .comments
                        .get(first_comment)
                        .is_some_and(|range| usize::from(range.start()) < guard_end);
                    if !has_comments
                        && let Some(run) = prepare_run(
                            self.source,
                            &items[start..end],
                            self.file_endings,
                            self.style,
                        )
                    {
                        self.runs.push(run);
                    }
                }
            }
            start = end;
        }
    }

    fn collect_module(&mut self, module: &ast::Module) {
        if !module.attrs_with_doc().all(|attribute| match attribute {
            ast::AnyAttr::DocComment(_) => true,
            ast::AnyAttr::Attr(attribute) => allows_module_body(&attribute),
        }) {
            return;
        }
        let Some(list) = module.item_list() else {
            return;
        };
        let (Some(open), Some(close)) = (list.l_curly_token(), list.r_curly_token()) else {
            return;
        };
        let items: Vec<_> = list.items().collect();
        self.collect_scope(
            &items,
            usize::from(open.text_range().end())..usize::from(close.text_range().start()),
            !list.attrs_with_doc().all(allows_inner_header),
        );
    }
}

fn allows_inner_header(attribute: ast::AnyAttr) -> bool {
    if !attribute.kind().is_inner() {
        return false;
    }
    let ast::AnyAttr::Attr(attribute) = attribute else {
        return true;
    };
    let Some(meta) = attribute.meta() else {
        return false;
    };
    if matches!(meta, ast::Meta::CfgMeta(_)) {
        return true;
    }
    let Some(path) = meta.path() else {
        return false;
    };
    if path.qualifier().is_some()
        || path
            .segment()
            .is_none_or(|segment| segment.coloncolon_token().is_some())
    {
        return false;
    }
    let Some(name) = path.as_single_name_ref() else {
        return false;
    };
    match meta {
        ast::Meta::KeyValueMeta(_) | ast::Meta::PathMeta(_) | ast::Meta::TokenTreeMeta(_)
            if name.text() == "doc" =>
        {
            true
        }
        ast::Meta::TokenTreeMeta(meta)
            if matches!(name.text(), "allow" | "warn" | "deny" | "forbid" | "expect") =>
        {
            meta.token_tree().is_some_and(|tokens| {
                tokens.l_paren_token().is_some() && tokens.r_paren_token().is_some()
            })
        }
        _ => false,
    }
}

fn allows_module_body(attribute: &ast::Attr) -> bool {
    match attribute.meta() {
        Some(ast::Meta::CfgMeta(_)) => true,
        Some(
            meta @ (ast::Meta::KeyValueMeta(_)
            | ast::Meta::PathMeta(_)
            | ast::Meta::TokenTreeMeta(_)),
        ) => meta.path().is_some_and(|path| {
            path.qualifier().is_none()
                && path.coloncolon_token().is_none()
                && path
                    .as_single_name_ref()
                    .is_some_and(|name| name.text() == "doc")
        }),
        _ => false,
    }
}

fn is_declaration(item: &ast::Item) -> bool {
    match item {
        ast::Item::Use(_) => true,
        ast::Item::Module(module) => module.item_list().is_none(),
        _ => false,
    }
}

fn inside_attribute(token: &SyntaxToken) -> bool {
    token.parent().is_some_and(|parent| {
        parent
            .ancestors()
            .any(|node| matches!(node.kind(), SyntaxKind::ATTR | SyntaxKind::DOC_COMMENT))
    })
}

fn item_content_range(item: &ast::Item) -> Option<TextRange> {
    // Leading comments can be attached to an item by the parser. Keep them in
    // the gap checked for comments, but exclude attributes owned by that item.
    let mut tokens = item
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia() && !inside_attribute(token));
    let first = tokens.next()?;
    let last = tokens.last().unwrap_or_else(|| first.clone());
    Some(TextRange::new(
        first.text_range().start(),
        last.text_range().end(),
    ))
}

fn module_declaration(item: &ast::Item) -> Option<Declaration> {
    let ast::Item::Module(module) = item else {
        return None;
    };
    if module.item_list().is_some() || module.attrs_with_doc().next().is_some() {
        return None;
    }
    let name = module.name()?.text().to_string();
    let keyword = module.mod_token()?;
    let semicolon = module.semicolon_token()?;
    let visibility = module.visibility();
    let start = visibility
        .as_ref()
        .map(|visibility| visibility.syntax().text_range().start())
        .unwrap_or_else(|| keyword.text_range().start());
    Some(Declaration {
        group: if visibility.is_some() {
            DeclarationGroup::PublicModule
        } else {
            DeclarationGroup::PrivateModule
        },
        range: usize::from(start)..usize::from(semicolon.text_range().end()),
        visibility: normalize_visibility(visibility)?,
        kind: DeclarationKind::Module(name),
    })
}

fn use_declaration(import: &ast::Use) -> Option<Declaration> {
    if import.attrs_with_doc().next().is_some() {
        return None;
    }
    let tree = import.use_tree()?;
    let mut parsed = ImportTree::parse(&tree, true, false)?;
    parsed.normalize();
    let group = match identifier_name(parsed.path.first()?) {
        "std" | "core" | "alloc" => DeclarationGroup::StandardUse,
        "crate" | "self" | "super" => DeclarationGroup::LocalUse,
        _ => DeclarationGroup::ExternalUse,
    };
    let keyword = import.use_token()?;
    let semicolon = import.semicolon_token()?;
    let visibility = import.visibility();
    let start = visibility
        .as_ref()
        .map(|visibility| visibility.syntax().text_range().start())
        .unwrap_or_else(|| keyword.text_range().start());
    Some(Declaration {
        group,
        range: usize::from(start)..usize::from(semicolon.text_range().end()),
        visibility: normalize_visibility(visibility)?,
        kind: DeclarationKind::Use(parsed),
    })
}

fn normalize_visibility(visibility: Option<ast::Visibility>) -> Option<String> {
    let Some(visibility) = visibility else {
        return Some(String::new());
    };
    let tokens: Vec<_> = visibility
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .map(|token| token.text().to_owned())
        .collect();
    if tokens == ["pub"] {
        return Some("pub".to_owned());
    }
    if tokens.len() < 4 || tokens[..2] != ["pub", "("] || tokens.last()? != ")" {
        return None;
    }
    let mut path = &tokens[2..tokens.len() - 1];
    if path.first()? == "in" {
        path = &path[1..];
    }
    let path = path.concat();
    let prefix = if matches!(path.as_str(), "crate" | "self" | "super") {
        ""
    } else {
        "in "
    };
    Some(format!("pub({prefix}{path})"))
}

fn merge_imports(declarations: Vec<Declaration>) -> Vec<Declaration> {
    let mut candidates = HashMap::<_, Vec<Declaration>>::new();
    let mut output = Vec::new();
    for declaration in declarations {
        if let DeclarationKind::Use(tree) = &declaration.kind {
            candidates
                .entry(tree.merge_key(&declaration.visibility))
                .or_default()
                .push(declaration);
        } else {
            output.push(declaration);
        }
    }
    for (key, mut candidate) in candidates {
        if candidate.len() < 2 {
            output.extend(candidate);
            continue;
        }
        let mut bindings = Vec::new();
        let mut prefix = Vec::new();
        for declaration in &candidate {
            if let DeclarationKind::Use(tree) = &declaration.kind {
                tree.collect_bindings(&mut prefix, &mut bindings);
            }
        }
        bindings.sort_by(|left, right| left.key().cmp(&right.key()));
        // Safety compares identifier names; construction needs exact spellings adjacent.
        let mut comparison_keys: Vec<_> =
            bindings.iter().map(ImportBinding::comparison_key).collect();
        comparison_keys.sort();
        let imports_root = bindings
            .iter()
            .any(|binding| binding.kind == ImportKind::Normal && binding.path == key.root);
        let incompatible = comparison_keys.windows(2).any(|pair| {
            let left = &pair[0];
            let right = &pair[1];
            left == right
                || (left.1 == ImportKind::Normal
                    && right.0.starts_with(&left.0)
                    && (left.0 != right.0 || right.1 != ImportKind::Normal))
        });
        if imports_root || incompatible {
            output.extend(candidate);
            continue;
        }
        let mut tree = ImportTree::from_bindings(&bindings, 0);
        tree.absolute = key.absolute;
        tree.normalize();
        let mut first = candidate.swap_remove(0);
        first.kind = DeclarationKind::Use(tree);
        output.push(first);
    }
    output
}

fn prepare_run<'a>(
    source: &'a str,
    items: &[ast::Item],
    file_endings: LineEndings,
    style: StyleEdition,
) -> Option<DeclarationRun<'a>> {
    let declarations: Vec<_> = items
        .iter()
        .map(|item| match item {
            ast::Item::Use(import) => use_declaration(import),
            _ => module_declaration(item),
        })
        .collect::<Option<_>>()?;
    if declarations.windows(2).any(|pair| {
        !source[pair[0].range.end..pair[1].range.start]
            .chars()
            .all(char::is_whitespace)
    }) {
        return None;
    }
    let range = declarations.first()?.range.start..declarations.last()?.range.end;
    let suffix = &source[range.end..];
    let spaces = suffix
        .bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .count();
    let ending_length = if suffix[spaces..].starts_with("\r\n") {
        2
    } else if suffix[spaces..].starts_with(['\n', '\r']) {
        1
    } else {
        0
    };
    let endings = line_endings(&source[range.start..range.end + spaces + ending_length]);
    let newline = match endings {
        LineEndings::Mixed => return None,
        LineEndings::CrLf => "\r\n",
        LineEndings::Lf => "\n",
        LineEndings::None => match file_endings {
            LineEndings::CrLf => "\r\n",
            _ => "\n",
        },
    };
    let line_prefix = source[..range.start].rsplit('\n').next()?;
    let indent = if line_prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        line_prefix
    } else {
        ""
    };
    let mut declarations = merge_imports(declarations);
    for declaration in &mut declarations {
        if let DeclarationKind::Use(tree) = &mut declaration.kind {
            tree.sort(style);
        }
    }
    declarations.sort_by(|left, right| compare_declarations(left, right, style));
    Some(DeclarationRun {
        range,
        declarations,
        newline,
        indent,
    })
}

fn compare_declarations(left: &Declaration, right: &Declaration, style: StyleEdition) -> Ordering {
    left.group
        .cmp(&right.group)
        .then_with(|| match (&left.kind, &right.kind) {
            (DeclarationKind::Use(left), DeclarationKind::Use(right)) => {
                compare_trees(left, right, style)
            }
            (DeclarationKind::Module(left), DeclarationKind::Module(right)) => {
                identifier_name(left).cmp(identifier_name(right))
            }
            _ => unreachable!("module and use declarations belong to different groups"),
        })
        .then_with(|| left.range.start.cmp(&right.range.start))
}

fn line_endings(source: &str) -> LineEndings {
    let bytes = source.as_bytes();
    let mut lf = false;
    let mut crlf = false;
    for (index, &byte) in bytes.iter().enumerate() {
        match byte {
            b'\n' if index > 0 && bytes[index - 1] == b'\r' => crlf = true,
            b'\n' => lf = true,
            b'\r' if bytes.get(index + 1) != Some(&b'\n') => return LineEndings::Mixed,
            _ => {}
        }
    }
    match (lf, crlf) {
        (true, true) => LineEndings::Mixed,
        (true, false) => LineEndings::Lf,
        (false, true) => LineEndings::CrLf,
        (false, false) => LineEndings::None,
    }
}
