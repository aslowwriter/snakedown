use std::path::PathBuf;

use url::Url;

use crate::render::formats::Renderer;

#[derive(Default)]
pub struct MdRenderer {}
impl MdRenderer {
    pub fn new() -> Self {
        Self {}
    }
}
impl Renderer for MdRenderer {
    fn render_header(&self, content: &str, level: usize) -> String {
        format!("{} {}", "#".repeat(level), content.trim())
    }

    fn render_front_matter(&self, title: Option<&str>) -> String {
        if let Some(t) = title {
            self.render_header(t, 1)
        } else {
            String::new()
        }
    }

    fn render_reference(&self, target: String, display_text: String) -> String {
        let t = if Url::parse(&target).is_ok() {
            target
        } else {
            format!("{}.md", target)
        };
        format!("[{display_text}]({t})")
    }

    fn content_path(&self) -> Option<PathBuf> {
        None
    }

    fn index_file(&self, _title: Option<String>) -> Option<(PathBuf, String)> {
        None
    }
}

#[cfg(test)]
mod test {

    use pretty_assertions::assert_eq;

    use color_eyre::Result;

    use crate::render::formats::{Renderer, md::MdRenderer};
    #[test]
    fn test_render_md_header() -> Result<()> {
        let text = String::from("foo");
        let out = MdRenderer::new().render_header(&text, 1);
        assert_eq!(out, String::from("# foo"));
        Ok(())
    }

    #[test]
    fn test_render_external_ref() -> Result<()> {
        let text = String::from("foo");
        let url = String::from("https://example.com/docs/foo/bar/baz.html#Bullshit");
        let out = MdRenderer::new().render_reference(url, text);
        assert_eq!(
            out,
            String::from("[foo](https://example.com/docs/foo/bar/baz.html#Bullshit)")
        );
        Ok(())
    }
    #[test]
    fn test_render_internal_ref() -> Result<()> {
        let text = String::from("foo");
        let rel_path = String::from("foo/bar/index");

        let out = MdRenderer::new().render_reference(rel_path, text);
        assert_eq!(out, String::from("[foo](foo/bar/index.md)"));
        Ok(())
    }
}
