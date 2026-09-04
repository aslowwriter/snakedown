use std::path::{Path, PathBuf};

pub mod md;
pub mod zola;

pub trait Renderer {
    fn render_header(&self, content: &str, level: usize) -> String;
    fn render_front_matter(&self, title: Option<&str>) -> String;
    fn render_source_location_link(
        &self,
        source_file: &Path,
        range: Option<(usize, usize)>,
    ) -> String;
    fn render_reference(&self, target: String, display_text: String) -> String;

    // This is on the Renderer because it is ssg specific.
    // e.g. zola places content in the `content` folder at the site root
    // but markdown places it just wherever it is pointed.
    fn content_path(&self) -> Option<PathBuf>;

    fn index_file(&self, title: Option<String>) -> Option<(PathBuf, String)>;
}

impl<T: Renderer + ?Sized> Renderer for &T {
    fn render_header(&self, content: &str, level: usize) -> String {
        (**self).render_header(content, level)
    }

    fn render_reference(&self, target: String, display_text: String) -> String {
        (**self).render_reference(target, display_text)
    }
    fn render_front_matter(&self, title: Option<&str>) -> String {
        (**self).render_front_matter(title)
    }

    fn content_path(&self) -> Option<PathBuf> {
        (**self).content_path()
    }
    fn index_file(&self, title: Option<String>) -> Option<(PathBuf, String)> {
        (**self).index_file(title)
    }

    fn render_source_location_link(
        &self,
        source_file: &Path,
        range: Option<(usize, usize)>,
    ) -> String {
        (**self).render_source_location_link(source_file, range)
    }
}

impl Renderer for Box<dyn Renderer> {
    fn render_header(&self, content: &str, level: usize) -> String {
        (**self).render_header(content, level)
    }
    fn render_front_matter(&self, title: Option<&str>) -> String {
        (**self).render_front_matter(title)
    }
    fn render_reference(&self, target: String, display_text: String) -> String {
        (**self).render_reference(target, display_text)
    }
    fn content_path(&self) -> Option<PathBuf> {
        (**self).content_path()
    }
    fn render_source_location_link(
        &self,
        source_file: &Path,
        range: Option<(usize, usize)>,
    ) -> String {
        (**self).render_source_location_link(source_file, range)
    }
    fn index_file(&self, title: Option<String>) -> Option<(PathBuf, String)> {
        (**self).index_file(title)
    }
}
