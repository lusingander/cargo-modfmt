use unicode_width::UnicodeWidthStr;

use crate::organize::imports::{ImportAlias, ImportTree, ImportTreeBody};

pub const IMPORT_WIDTH: usize = 100;
const INDENT_WIDTH: usize = 4;

impl ImportTree {
    fn write_inline(&self, output: &mut String) {
        if self.absolute {
            output.push_str("::");
        }
        output.push_str(&self.path.join("::"));
        if matches!(self.body, ImportTreeBody::Glob) {
            if !self.path.is_empty() {
                output.push_str("::");
            }
            output.push('*');
            return;
        }
        let ImportTreeBody::Group(children) = &self.body else {
            match self.alias() {
                ImportAlias::None => {}
                ImportAlias::Named(name) => {
                    output.push_str(" as ");
                    output.push_str(name);
                }
                ImportAlias::Underscore => output.push_str(" as _"),
            }
            return;
        };
        output.push_str("::{");
        for (index, child) in children.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            child.write_inline(output);
        }
        output.push('}');
    }

    pub fn write_layout(&self, output: &mut String, width: usize, indent: &str, newline: &str) {
        let ImportTreeBody::Group(children) = &self.body else {
            self.write_inline(output);
            return;
        };
        let mut prefix = String::new();
        if self.absolute {
            prefix.push_str("::");
        }
        prefix.push_str(&self.path.join("::"));
        prefix.push_str("::");
        let child_indent = format!("{indent}{}", " ".repeat(INDENT_WIDTH));
        let child_width = IMPORT_WIDTH.saturating_sub(column_after(0, &child_indent) + 1);
        let rendered: Vec<_> = children
            .iter()
            .map(|child| {
                let mut text = String::new();
                child.write_layout(&mut text, child_width, &child_indent, newline);
                text
            })
            .collect();
        let nested = children
            .iter()
            .any(|child| matches!(child.body, ImportTreeBody::Group(_)));
        // Rustfmt reserves both braces and two additional columns for a list.
        let remaining = if nested {
            0
        } else {
            width.saturating_sub(prefix.len() + 4)
        };
        let total_width = rendered.iter().map(|text| text.width()).sum::<usize>()
            + 2 * children.len().saturating_sub(1);
        let horizontal =
            total_width <= remaining && !rendered.iter().any(|text| text.contains('\n'));
        let mut list = String::new();
        let mut line_width = 0;
        let mut previous_path = false;
        let mut trailing_separator = false;
        for (index, text) in rendered.iter().enumerate() {
            if horizontal {
                if index != 0 {
                    list.push_str(", ");
                }
                list.push_str(text);
                continue;
            }
            let item_width =
                text.width() + usize::from(index + 1 < rendered.len() || trailing_separator);
            if (line_width > 0 && line_width + 1 + item_width > child_width)
                || (nested && (previous_path || (index != 0 && text.contains("::"))))
            {
                list.push_str(newline);
                list.push_str(&child_indent);
                line_width = 0;
                trailing_separator = true;
            } else if line_width > 0 {
                list.push(' ');
                line_width += 1;
            }
            list.push_str(text);
            list.push(',');
            line_width += item_width;
            previous_path = text.contains("::");
        }
        let multiline = list.contains('\n') || list.len() > remaining;
        output.push_str(&prefix);
        output.push('{');
        if multiline {
            output.push_str(newline);
            output.push_str(&child_indent);
        }
        output.push_str(&list);
        if multiline {
            output.push_str(newline);
            output.push_str(indent);
        }
        output.push('}');
    }
}

pub fn column_after(column: usize, text: &str) -> usize {
    text.chars()
        .fold(column, |column, character| match character {
            '\r' | '\n' => 0,
            '\t' => column + INDENT_WIDTH - column % INDENT_WIDTH,
            _ => column + character.len_utf8(),
        })
}
