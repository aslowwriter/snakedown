use color_eyre::Result;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tera::Context;

use crate::indexing::validated::ValidatedIndex;
use crate::render::formats::Renderer;
use crate::render::jupyter::{RenderedNotebook, render_notebook};
use crate::render::render_object;

pub struct SerializableIndex {
    pub page_store: HashMap<PathBuf, String>,
    pub notebook_store: HashMap<PathBuf, RenderedNotebook>,
}

impl SerializableIndex {
    /// this is where al the rendering happens
    pub fn from_validated<R: Renderer>(
        validated: ValidatedIndex,
        renderer: &R,
        ctx: &Context,
        notebook_out_path: Option<&Path>,
    ) -> Self {
        let mut page_store = HashMap::new();
        validated.page_store.iter().for_each(|(key, page)| {
            let rendered_page = render_object(&page.content, key.to_string(), renderer, ctx);

            page_store.insert(PathBuf::from(key), rendered_page);
        });
        let mut notebook_store = HashMap::new();
        if let Some(nb_out_path) = notebook_out_path {
            validated.notebook_store.iter().for_each(|(key, notebook)| {
                if let Ok(mut rendered_notebook) = render_notebook(
                    nb_out_path
                        .join(key)
                        .file_stem()
                        .map(|p| p.display().to_string())
                        .as_deref(),
                    notebook,
                    renderer,
                ) {
                    // some tools insert an extra EOL at the end of the file
                    if !rendered_notebook.text.ends_with("\n") {
                        rendered_notebook.text.push('\n');
                    }
                    notebook_store.insert(PathBuf::from(key), rendered_notebook);
                }
            });
        }
        SerializableIndex {
            page_store,
            notebook_store,
        }
    }

    fn serialize_pages(&self, pages_out_path: &Path) -> Result<()> {
        create_dir_all(pages_out_path)?;

        for (key, page) in self.page_store.iter() {
            let file_path = pages_out_path.join(key).with_added_extension("md");
            let mut file = File::create(file_path)?;
            file.write_all(page.as_bytes())?;
        }

        Ok(())
    }

    fn serialize_notebooks(&self, notebook_out_path: PathBuf) -> Result<()> {
        create_dir_all(&notebook_out_path)?;
        for (key, rendered) in self.notebook_store.iter() {
            let dir_path = notebook_out_path.join(key);
            let file_path = dir_path.clone().join("index").with_added_extension("md");

            create_dir_all(dir_path.clone())?;
            let mut file = File::create(file_path)?;
            file.write_all(rendered.text.as_bytes())?;
            for img in rendered.images.clone() {
                let mut img_file = File::create(dir_path.join(img.name))?;
                img_file.write_all(&img.data)?;
            }
        }

        Ok(())
    }

    pub fn serialize(
        self,
        pages_out_path: &Path,
        notebook_out_path: Option<PathBuf>,
    ) -> Result<()> {
        self.serialize_pages(pages_out_path)?;
        if let Some(out_root) = notebook_out_path {
            self.serialize_notebooks(out_root)?;
        }
        Ok(())
    }
}
