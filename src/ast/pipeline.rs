use std::collections::HashMap;

use crate::types::MarkdownOptions;

use super::extractor::{TranslatableNode, extract_translatable_nodes};
use super::parser::{MarkdownAst, parse_markdown_with_options};
use super::render::{RenderError, replace_translated_nodes_and_render};

#[derive(Debug, Clone)]
pub struct AstTranslationPipeline {
    pub ast: MarkdownAst,
    pub nodes: Vec<TranslatableNode>,
}

impl AstTranslationPipeline {
    pub fn parse(
        markdown: &str,
        options: comrak::Options<'static>,
        markdown_options: &MarkdownOptions,
    ) -> Self {
        let ast = parse_markdown_with_options(markdown, options);
        let nodes = extract_translatable_nodes(&ast, markdown_options);
        Self { ast, nodes }
    }

    #[allow(clippy::result_large_err)]
    pub fn render_with_translations(
        &self,
        markdown_options: &MarkdownOptions,
        translations: &HashMap<usize, String>,
    ) -> Result<String, RenderError> {
        replace_translated_nodes_and_render(&self.ast, markdown_options, translations)
    }
}

pub fn parse_extract_translate_replace_render(
    markdown: &str,
    options: comrak::Options<'static>,
    markdown_options: &MarkdownOptions,
    translations: &HashMap<usize, String>,
) -> Result<String, RenderError> {
    let pipeline = AstTranslationPipeline::parse(markdown, options, markdown_options);
    pipeline.render_with_translations(markdown_options, translations)
}
