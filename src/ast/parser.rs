use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};

#[derive(Debug, Clone)]
pub struct MarkdownAst {
    pub(crate) markdown: String,
    pub(crate) options: Options<'static>,
}

impl MarkdownAst {
    pub fn contains_value(&self, matcher: impl Fn(&NodeValue) -> bool) -> bool {
        self.with_arena_root(|root| contains_value(root, &matcher))
    }

    pub fn with_root<R>(&self, visitor: impl for<'a> FnOnce(&'a AstNode<'a>) -> R) -> R {
        self.with_arena_root(visitor)
    }

    fn with_arena_root<R>(&self, visitor: impl for<'a> FnOnce(&'a AstNode<'a>) -> R) -> R {
        let arena = Arena::new();
        let root = parse_document(&arena, &self.markdown, &self.options);
        visitor(root)
    }
}

fn contains_value<'a>(node: &'a AstNode<'a>, matcher: &impl Fn(&NodeValue) -> bool) -> bool {
    if matcher(&node.data.borrow().value) {
        return true;
    }

    node.children()
        .any(move |child| contains_value(child, matcher))
}

pub fn parse_markdown(input: &str) -> MarkdownAst {
    parse_markdown_with_options(input, markdown_parse_options())
}

pub fn parse_markdown_with_options(input: &str, options: Options<'static>) -> MarkdownAst {
    MarkdownAst {
        markdown: input.to_owned(),
        options,
    }
}

pub fn markdown_parse_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.front_matter_delimiter = Some("---".to_owned());
    options.extension.math_dollars = true;
    options
}
