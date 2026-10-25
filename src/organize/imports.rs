use ra_ap_syntax::ast::HasName;
use ra_ap_syntax::{AstNode, SyntaxKind, ast};

pub struct ImportTree {
    pub path: Vec<String>,
    pub absolute: bool,
    pub body: ImportTreeBody,
    pub origin: usize,
}

pub enum ImportTreeBody {
    Leaf(ImportAlias),
    Glob,
    // The written path includes an explicit final `self` segment.
    SelfImport(ImportAlias),
    // A qualifier-only path such as `crate as Root` or `self as Here`.
    ModuleImport(ImportAlias),
    Group(Vec<ImportTree>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportAlias {
    None,
    Named(String),
    Underscore,
}

impl ImportAlias {
    fn parse(rename: Option<ast::Rename>) -> Option<Self> {
        let Some(rename) = rename else {
            return Some(Self::None);
        };
        rename.as_token()?;
        if rename.underscore_token().is_some() {
            return Some(Self::Underscore);
        }
        let name = rename.name()?.text().to_string();
        Some(Self::Named(name))
    }

    pub fn comparison_key(&self) -> (u8, &str) {
        match self {
            Self::None => (0, ""),
            Self::Named(name) => (1, identifier_name(name)),
            Self::Underscore => (2, ""),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportKind {
    Normal,
    TypeOnly,
    Glob,
}

pub struct ImportBinding {
    pub path: Vec<String>,
    pub kind: ImportKind,
    pub alias: ImportAlias,
    pub origin: usize,
}

impl ImportBinding {
    pub fn key(&self) -> (&[String], ImportKind, &ImportAlias) {
        (&self.path, self.kind, &self.alias)
    }

    pub fn comparison_key(&self) -> (Vec<&str>, ImportKind, (u8, &str)) {
        (
            path_names(&self.path).collect(),
            self.kind,
            self.alias.comparison_key(),
        )
    }
}

impl ImportTree {
    pub fn parse(tree: &ast::UseTree, is_root: bool, parent_has_identifier: bool) -> Option<Self> {
        let glob = tree.star_token().is_some();
        let alias = ImportAlias::parse(tree.rename())?;
        let list = tree.use_tree_list();
        let (path, absolute) = match tree.path() {
            Some(path) => {
                let absolute = path.first_segment()?.coloncolon_token().is_some();
                if absolute && !is_root {
                    return None;
                }
                (
                    use_path(&path, is_root && !absolute, list.is_some() || glob)?,
                    absolute,
                )
            }
            None if glob && !is_root && tree.coloncolon_token().is_none() => (Vec::new(), false),
            None => return None,
        };
        let has_identifier = parent_has_identifier
            || path
                .iter()
                .any(|part| !matches!(part.as_str(), "crate" | "self" | "super"));
        let body = match list {
            _ if glob => {
                if alias != ImportAlias::None {
                    return None;
                }
                ImportTreeBody::Glob
            }
            Some(list) => {
                if alias != ImportAlias::None {
                    return None;
                }
                let children: Vec<_> = list
                    .use_trees()
                    .map(|child| Self::parse(&child, false, has_identifier))
                    .collect::<Option<_>>()?;
                if children.is_empty() {
                    return None;
                }
                ImportTreeBody::Group(children)
            }
            None if path.last()?.as_str() == "self" && (!is_root || path.len() > 1) => {
                if !has_identifier && alias == ImportAlias::None {
                    return None;
                }
                ImportTreeBody::SelfImport(alias)
            }
            None if !has_identifier => {
                if alias == ImportAlias::None {
                    return None;
                }
                ImportTreeBody::ModuleImport(alias)
            }
            None => ImportTreeBody::Leaf(alias),
        };
        Some(Self {
            path,
            absolute,
            body,
            origin: usize::from(tree.syntax().text_range().start()),
        })
    }

    pub fn alias(&self) -> &ImportAlias {
        match &self.body {
            ImportTreeBody::Leaf(alias)
            | ImportTreeBody::SelfImport(alias)
            | ImportTreeBody::ModuleImport(alias) => alias,
            ImportTreeBody::Group(_) | ImportTreeBody::Glob => &ImportAlias::None,
        }
    }

    pub fn normalize(&mut self) {
        match &mut self.body {
            ImportTreeBody::Leaf(alias @ ImportAlias::Named(_)) => {
                if let ImportAlias::Named(name) = alias
                    && self
                        .path
                        .last()
                        .is_some_and(|last| identifier_name(last) == identifier_name(name))
                {
                    *alias = ImportAlias::None;
                }
            }
            ImportTreeBody::SelfImport(_) if self.path.len() > 1 => {
                let body = std::mem::replace(&mut self.body, ImportTreeBody::Glob);
                self.path.pop();
                self.body = ImportTreeBody::Group(vec![Self {
                    path: vec!["self".to_owned()],
                    absolute: false,
                    body,
                    origin: self.origin,
                }]);
            }
            ImportTreeBody::Group(children) => {
                for child in children.iter_mut() {
                    child.normalize();
                }
                if children.len() == 1 && !children[0].is_direct_self() {
                    let child = children.pop().unwrap();
                    self.path.extend(child.path);
                    self.body = child.body;
                }
            }
            _ => {}
        }
    }

    fn is_direct_self(&self) -> bool {
        matches!(self.body, ImportTreeBody::SelfImport(_)) && self.path == ["self"]
    }

    pub fn collect_bindings(&self, prefix: &mut Vec<String>, bindings: &mut Vec<ImportBinding>) {
        let start = prefix.len();
        prefix.extend(self.path.iter().cloned());
        match &self.body {
            ImportTreeBody::Group(children) => {
                for child in children {
                    child.collect_bindings(prefix, bindings);
                }
            }
            body => {
                let kind = match body {
                    ImportTreeBody::SelfImport(_) => {
                        prefix.pop();
                        ImportKind::TypeOnly
                    }
                    ImportTreeBody::ModuleImport(_) => ImportKind::TypeOnly,
                    ImportTreeBody::Glob => ImportKind::Glob,
                    _ => ImportKind::Normal,
                };
                bindings.push(ImportBinding {
                    path: prefix.clone(),
                    kind,
                    alias: self.alias().clone(),
                    origin: self.origin,
                });
            }
        }
        prefix.truncate(start);
    }

    // Bindings are sorted, compatible, and cannot import an ordinary merge root.
    pub fn from_bindings(bindings: &[ImportBinding], depth: usize) -> Self {
        if bindings.len() == 1 && bindings[0].kind != ImportKind::TypeOnly {
            return Self {
                path: bindings[0].path[depth..].to_vec(),
                absolute: false,
                body: if bindings[0].kind == ImportKind::Glob {
                    ImportTreeBody::Glob
                } else {
                    ImportTreeBody::Leaf(bindings[0].alias.clone())
                },
                origin: bindings[0].origin,
            };
        }
        let shared = bindings[0].path[depth..]
            .iter()
            .zip(&bindings[bindings.len() - 1].path[depth..])
            .take_while(|(left, right)| left == right)
            .count();
        let mut end = depth + shared;
        if bindings[0].path.len() == end && bindings[0].kind == ImportKind::Normal {
            // Ordinary aliases must remain siblings under their parent, never `self` imports.
            end -= 1;
        }
        let path = bindings[0].path[depth..end].to_vec();
        let mut children = Vec::new();
        let mut start = 0;
        while start < bindings.len() && bindings[start].path.len() == end {
            let binding = &bindings[start];
            children.push(Self {
                absolute: false,
                path: if binding.kind == ImportKind::Glob {
                    Vec::new()
                } else {
                    vec!["self".to_string()]
                },
                body: if binding.kind == ImportKind::Glob {
                    ImportTreeBody::Glob
                } else {
                    ImportTreeBody::SelfImport(binding.alias.clone())
                },
                origin: bindings[start].origin,
            });
            start += 1;
        }
        while start < bindings.len() {
            let mut next = start + 1;
            while next < bindings.len() && bindings[next].path[end] == bindings[start].path[end] {
                next += 1;
            }
            let branch = &bindings[start..next];
            if branch[0].kind == ImportKind::Normal && branch[0].path.len() == end + 1 {
                children.extend(branch.iter().map(|binding| Self {
                    path: binding.path[end..].to_vec(),
                    absolute: false,
                    body: ImportTreeBody::Leaf(binding.alias.clone()),
                    origin: binding.origin,
                }));
            } else {
                children.push(Self::from_bindings(branch, end));
            }
            start = next;
        }
        Self {
            path,
            absolute: false,
            body: ImportTreeBody::Group(children),
            origin: bindings.iter().map(|binding| binding.origin).min().unwrap(),
        }
    }
}

fn use_path(path: &ast::Path, allows_qualifiers: bool, has_children: bool) -> Option<Vec<String>> {
    let mut parts: Vec<String> = Vec::new();
    let mut allows_super = allows_qualifiers;
    let mut segments = path.segments().peekable();
    while let Some(segment) = segments.next() {
        let mut tokens = segment
            .syntax()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter(|token| !token.kind().is_trivia());
        let token = tokens.next()?;
        let token = if token.kind() == SyntaxKind::COLON2 && parts.is_empty() {
            tokens.next()?
        } else {
            token
        };
        if tokens.next().is_some() {
            return None;
        }
        match token.kind() {
            SyntaxKind::IDENT => allows_super = false,
            SyntaxKind::CRATE_KW if allows_qualifiers && parts.is_empty() => allows_super = false,
            SyntaxKind::SELF_KW if allows_qualifiers && parts.is_empty() => {}
            SyntaxKind::SELF_KW if !has_children && segments.peek().is_none() => {
                allows_super = false;
            }
            SyntaxKind::SUPER_KW if allows_super => {}
            _ => return None,
        }
        parts.push(token.text().to_string());
    }
    (!parts.is_empty()).then_some(parts)
}

pub fn identifier_name(spelling: &str) -> &str {
    spelling.strip_prefix("r#").unwrap_or(spelling)
}

fn path_names(path: &[String]) -> impl Iterator<Item = &str> {
    path.iter().map(|part| identifier_name(part))
}
