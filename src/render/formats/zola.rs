use std::path::{Path, PathBuf};

use color_eyre::Result;
use url::Url;

use crate::render::formats::Renderer;

#[derive(Default)]
pub struct ZolaRenderer {}

impl Renderer for ZolaRenderer {
    fn render_header(&self, content: &str, level: usize) -> String {
        format!("{} {}", "#".repeat(level), content.trim())
    }

    fn render_front_matter(&self, title: Option<&str>) -> String {
        let mut out = String::new();
        out.push_str("+++\n");
        if let Some(t) = title {
            out.push_str(&format!("title = \"{t}\"\n"));
        };
        out.push_str("+++");
        out
    }

    fn render_reference(
        &self,
        display_text: Option<String>,
        target_prefix: &Path,
        target: String,
    ) -> Result<String> {
        let t = if Url::parse(&target).is_ok() {
            target
        } else {
            format!(
                "{}",
                PathBuf::from("@")
                    .join(target_prefix)
                    .join(target)
                    .with_added_extension("md")
                    .display()
            )
        };
        let rendered = match display_text {
            Some(text) => format!("[{text}]({t})"),
            None => format!("[{t}]({t})"),
        };
        Ok(rendered)
    }
    fn content_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from("content"))
    }

    fn index_file(&self, title: Option<String>) -> Option<(PathBuf, String)> {
        let index_front_matter = self.render_front_matter(title.as_deref());

        Some((PathBuf::from("_index.md"), index_front_matter))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_zola_header() -> Result<()> {
        let renderer = ZolaRenderer {};
        let obj_name = String::from("foo.bar.nasty-names_with_underscores_and_emoji_🙈");
        assert_eq!(
            renderer.render_header(&obj_name, 2),
            "## foo.bar.nasty-names_with_underscores_and_emoji_🙈"
        );
        Ok(())
    }

    #[test]
    fn test_empty_zola_front_matter() -> Result<()> {
        assert_eq!(
            ZolaRenderer {}.render_front_matter(None),
            r"+++
+++"
        );
        Ok(())
    }

    #[test]
    fn zola_internal_link_no_shortcode() -> Result<()> {
        #[cfg(windows)]
        let expected = r#"[Baz](@\foo.bar.baz.index.md)"#;
        #[cfg(not(windows))]
        let expected = r#"[Baz](@/foo.bar.baz.index.md)"#;

        assert_eq!(
            ZolaRenderer {}.render_reference(
                Some("Baz".to_string()),
                &PathBuf::from(""),
                String::from("foo.bar.baz.index")
            )?,
            expected
        );
        Ok(())
    }
    #[test]
    fn zola_external_link_no_shortcode() -> Result<()> {
        assert_eq!(
            ZolaRenderer {}.render_reference(
                Some("Dataset".to_string()),
                &PathBuf::from(""),
                "https://docs.xarray.dev/en/stable/generated/xarray.Dataset.html#xarray.Dataset"
                    .to_string(),
            )?,
            r#"[Dataset](https://docs.xarray.dev/en/stable/generated/xarray.Dataset.html#xarray.Dataset)"#
        );
        Ok(())
    }
    #[test]
    fn test_zola_front_matter_with_title() -> Result<()> {
        assert_eq!(
            ZolaRenderer {}.render_front_matter(Some("foo")),
            r#"+++
title = "foo"
+++"#
        );
        Ok(())
    }
}
