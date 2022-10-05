use super::parser::{Block, DocumentRoot, Inline, InlineBlock};

#[derive(Clone, Debug)]
pub struct Renderer {
    root: DocumentRoot,
}

impl Renderer {
    pub fn new(root: DocumentRoot) -> Self {
        Self { root }
    }

    pub fn render(&self) -> String {
        let mut builder = String::new();

        for block in &self.root.0 {
            render_block(&mut builder, block);
        }

        builder
    }
}

fn render_block(dst: &mut String, block: &Block) {
    match block {
        Block::H1(inner) => {
            dst.push_str("<h1>");
            render_inline(dst, inner);
            dst.push_str("</h1>");
        }
        Block::H2(inner) => {
            dst.push_str("<h2>");
            render_inline(dst, inner);
            dst.push_str("</h2>");
        }
        Block::H3(inner) => {
            dst.push_str("<h3>");
            render_inline(dst, inner);
            dst.push_str("</h3>");
        }
        Block::H4(inner) => {
            dst.push_str("<h4>");
            render_inline(dst, inner);
            dst.push_str("</h4>");
        }
        Block::H5(inner) => {
            dst.push_str("<h5>");
            render_inline(dst, inner);
            dst.push_str("</h5>");
        }
        Block::H6(inner) => {
            dst.push_str("<h6>");
            render_inline(dst, inner);
            dst.push_str("</h6>");
        }
        Block::P(inner) => {
            dst.push_str("<p>");
            render_inline(dst, inner);
            dst.push_str("</p>");
        }
        Block::Inline(inner) => {
            render_inline(dst, inner);
        }
        Block::Text(inner) => {
            render_text(dst, inner);
        }
    }
}

fn render_inline(dst: &mut String, elem: &Inline) {
    for block in &elem.0 {
        render_inline_block(dst, block);
    }
}

fn render_inline_block(dst: &mut String, elem: &InlineBlock) {
    match elem {
        InlineBlock::Strong(inner) => {
            dst.push_str("<strong>");
            render_inline(dst, inner);
            dst.push_str("</strong>");
        }
        InlineBlock::Emphasis(inner) => {
            dst.push_str("<em>");
            render_inline(dst, inner);
            dst.push_str("</em>");
        }
        InlineBlock::Text(inner) => {
            render_text(dst, inner);
        }
    }
}

fn render_text(dst: &mut String, elem: &str) {
    dst.push_str(elem);
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_build() {}
}
