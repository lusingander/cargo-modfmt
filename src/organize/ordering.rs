use std::cmp::Ordering;

use crate::organize::{
    StyleEdition,
    imports::{ImportBinding, ImportTree, ImportTreeBody, identifier_name},
};

impl StyleEdition {
    fn compare_names(self, left: &str, right: &str) -> Ordering {
        if left.is_empty() || right.is_empty() {
            left.cmp(right)
        } else if matches!(self, Self::E2024) {
            version_compare(identifier_name(left), identifier_name(right))
        } else {
            let key = |name: &str| {
                (
                    name.chars()
                        .all(|ch| ch.is_uppercase() || ch == '_' || ch.is_numeric()),
                    name.starts_with(char::is_uppercase),
                )
            };
            key(left).cmp(&key(right)).then_with(|| left.cmp(right))
        }
    }
}

impl ImportTree {
    pub fn sort(&mut self, style: StyleEdition) {
        if !matches!(self.body, ImportTreeBody::Group(_)) {
            return;
        }
        let mut bindings = Vec::new();
        self.collect_bindings(&mut Vec::new(), &mut bindings);
        let mut keys: Vec<_> = bindings.iter().map(ImportBinding::comparison_key).collect();
        keys.sort();
        // Adjacent equal aliases would be removed by rustfmt's list normalization.
        let sort_aliases = !keys.windows(2).any(|pair| pair[0] == pair[1]);
        self.sort_children(sort_aliases, style);
    }

    fn sort_children(&mut self, sort_aliases: bool, style: StyleEdition) {
        let ImportTreeBody::Group(children) = &mut self.body else {
            return;
        };
        for child in children.iter_mut() {
            child.sort_children(sort_aliases, style);
        }
        children.sort_by(|left, right| compare_children(left, right, sort_aliases, style));
    }
}

fn compare_children(
    left: &ImportTree,
    right: &ImportTree,
    sort_aliases: bool,
    style: StyleEdition,
) -> Ordering {
    compare_trees(left, right, style)
        .then_with(|| {
            if sort_aliases {
                left.alias()
                    .comparison_key()
                    .cmp(&right.alias().comparison_key())
            } else {
                Ordering::Equal
            }
        })
        .then_with(|| left.origin.cmp(&right.origin))
}

enum SortSegment<'a> {
    Name(&'a str),
    Glob,
    Group(&'a [ImportTree]),
}

impl SortSegment<'_> {
    fn rank(&self) -> u8 {
        match self {
            Self::Name("self") => 0,
            Self::Name("super") => 1,
            Self::Name("crate") => 2,
            Self::Name(_) => 3,
            Self::Glob => 4,
            Self::Group(_) => 5,
        }
    }

    fn compare(&self, other: &Self, style: StyleEdition) -> Ordering {
        self.rank()
            .cmp(&other.rank())
            .then_with(|| match (self, other) {
                (Self::Name(left), Self::Name(right)) => style.compare_names(left, right),
                (Self::Group(left), Self::Group(right)) => {
                    for (left, right) in left.iter().zip(right.iter()) {
                        let order = compare_trees(left, right, style);
                        if order != Ordering::Equal {
                            return order;
                        }
                    }
                    left.len().cmp(&right.len())
                }
                _ => Ordering::Equal,
            })
    }
}

fn sort_segments(tree: &ImportTree) -> impl Iterator<Item = SortSegment<'_>> {
    let absolute = tree.absolute.then_some(SortSegment::Name(""));
    let tail = match &tree.body {
        ImportTreeBody::Glob => Some(SortSegment::Glob),
        ImportTreeBody::Group(children) => Some(SortSegment::Group(children)),
        _ => None,
    };
    absolute
        .into_iter()
        .chain(tree.path.iter().map(|name| SortSegment::Name(name)))
        .chain(tail)
}

pub fn compare_trees(left: &ImportTree, right: &ImportTree, style: StyleEdition) -> Ordering {
    let mut left = sort_segments(left);
    let mut right = sort_segments(right);
    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) => {
                let order = left.compare(&right, style);
                if order != Ordering::Equal {
                    return order;
                }
            }
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return Ordering::Equal,
        }
    }
}

struct VersionChunk<'a> {
    text: &'a str,
    number: Option<usize>,
}

fn version_chunk<'a>(remaining: &mut &'a str) -> Option<VersionChunk<'a>> {
    let first = remaining.chars().next()?;
    let length = if first == '_' {
        1
    } else {
        remaining
            .char_indices()
            .skip(1)
            .find(|(_, ch)| *ch == '_' || ch.is_ascii_digit() != first.is_ascii_digit())
            .map_or(remaining.len(), |(index, _)| index)
    };
    let (text, rest) = remaining.split_at(length);
    *remaining = rest;
    let number = if first.is_ascii_digit() {
        // Match rustfmt's machine-sized numeric chunks, including overflow termination.
        Some(text.parse().ok()?)
    } else {
        None
    };
    Some(VersionChunk { text, number })
}

fn version_compare(mut left: &str, mut right: &str) -> Ordering {
    let mut padding = Ordering::Equal;
    loop {
        let (left, right) = match (version_chunk(&mut left), version_chunk(&mut right)) {
            (Some(left), Some(right)) => (left, right),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (None, None) => return padding,
        };
        let order = match (left.text, right.text, left.number, right.number) {
            ("_", "_", _, _) => Ordering::Equal,
            ("_", _, _, _) => Ordering::Less,
            (_, "_", _, _) => Ordering::Greater,
            (_, _, Some(a), Some(b)) => {
                if padding == Ordering::Equal {
                    let zeros = |text: &str| text.bytes().take_while(|byte| *byte == b'0').count();
                    padding = zeros(right.text).cmp(&zeros(left.text));
                }
                a.cmp(&b)
            }
            _ => left.text.cmp(right.text),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
}
