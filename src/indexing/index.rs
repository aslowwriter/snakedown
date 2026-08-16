use crate::render_notebook;
use crate::render_object;
use crate::{
    config::ExternalIndex,
    parsing::{
        ObjectDocumentation,
        python::{
            class::ClassDocumentation,
            function::FunctionDocumentation,
            jupyter::parse_notebook_file,
            module::{ModuleDocumentation, extract_module_documentation},
            utils::parse_python_file,
        },
    },
    render::formats::Renderer,
    should_include_reference,
};
use color_eyre::{Report, Result, eyre::eyre};
use edit_distance::edit_distance;
use nbformat::v4::Cell;
use sphinx_inv::SphinxInventoryReader;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tera::Context;
use tracing::warn;
use url::Url;

// TODO: I think the winning strategy here is to start with a `RawIndex` that is more or less the
// current index struct, which refers to all the original python stuff on disk with all the type
// info etc.
// then we do something like `let serializable_index = index.process()?;` with something like
// `RawIndex::process(self) -> SerializableIndex` which is a new object with everything
// preprocessed, incl stripped prefixes, all references validated and expanded etc.
// then later we can just write everything to disk separately. That's a nice separation of concerns
// see also: https://github.com/aslowwriter/snakedown/issues/57

#[derive(Debug)]
pub struct RawIndex {
    pub pkg_name: String,
    pub internal_object_store: HashMap<String, ObjectDocumentation>,
    pub external_object_store: HashMap<String, Url>,
    pub notebook_store: HashMap<String, Vec<Cell>>,
    pub skip_undoc: bool,
    pub skip_private: bool,
    pub pkg_root: PathBuf,
}

impl RawIndex {
    pub fn new(pkg_root: PathBuf, skip_undoc: bool, skip_private: bool) -> Result<Self> {
        let pkg_name = pkg_root
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .ok_or(eyre!("Error determining pkg_root name"))?;
        Ok(Self {
            pkg_name,
            internal_object_store: HashMap::new(),
            external_object_store: HashMap::new(),
            notebook_store: HashMap::new(),
            pkg_root,
            skip_undoc,
            skip_private,
        })
    }

    pub fn index_file(&mut self, path: PathBuf) -> Result<()> {
        tracing::info!("Indexing {}", &path.display());

        let parsed = parse_python_file(&path);

        let rel_module_file_path = path.clone().strip_prefix(&self.pkg_root)?.to_path_buf();
        let module_import_path: String = {
            let tmp_module_path =
                get_from_import_path(self.pkg_name.clone(), &rel_module_file_path)?;
            tmp_module_path
                .strip_suffix(".__init__")
                .unwrap_or(&tmp_module_path)
                .to_string()
        };

        match parsed {
            Ok(contents) => {
                let mod_docs =
                    extract_module_documentation(&contents, self.skip_private, self.skip_undoc);
                if should_include_module(&mod_docs, self.skip_undoc) {
                    self.internal_object_store.insert(
                        module_import_path.clone(),
                        ObjectDocumentation::Module(mod_docs.clone()),
                    );
                    for class_docs in &mod_docs.classes {
                        if should_include_class(class_docs, self.skip_private, self.skip_undoc) {
                            index_class(self, class_docs, module_import_path.clone())?;
                        }
                    }

                    for function_docs in mod_docs.functions {
                        if should_include_function(
                            &function_docs,
                            self.skip_private,
                            self.skip_undoc,
                        ) {
                            index_functions(self, &function_docs, module_import_path.clone())?;
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "The following error odducred while processing {}: {}",
                    &path.display(),
                    e
                );
                Err(e)
            }
        }
    }

    pub fn index_notebook(&mut self, path: &Path) -> Result<()> {
        let notebook_name = path
            .file_stem()
            .ok_or(eyre!("Could not deternime file stem"))?
            .to_str()
            .ok_or(eyre!("Could not convert file stem to string"))?
            .to_string();
        let notebook_contents = parse_notebook_file(path)?;

        if self
            .notebook_store
            .insert(notebook_name.clone(), notebook_contents)
            .is_some()
        {
            warn!("overwriting notebook called {notebook_name}")
        }
        Ok(())
    }

    pub fn load_external_references(
        &mut self,
        externals: HashMap<String, ExternalIndex>,
        cache_path: &Path,
        permissive: bool,
    ) -> Result<()> {
        for (key, ext_index) in externals {
            let inv_path = cache_path.join("sphinx").join(key).with_extension("inv");

            // TODO: This will be made more flexible once we add a permissive mode
            // see https://github.com/aslowwriter/snakedown/issues/38
            if !inv_path.exists() && permissive {
                continue;
            }
            let external_base_url = Url::parse(&ext_index.url)?;

            let reference_reader = SphinxInventoryReader::from_path(&inv_path)?;
            for maybe_ref in reference_reader {
                let r = maybe_ref?;
                if !should_include_reference(&r) {
                    continue;
                }
                let expanded_location = &r.expanded_location();
                self.external_object_store
                    .insert(r.name, external_base_url.clone().join(expanded_location)?);
            }
        }

        Ok(())
    }

    pub fn validate_references(&self) -> Result<(), Vec<Report>> {
        let mut errors: Vec<_> = Vec::new();
        for (key, obj) in self.internal_object_store.iter() {
            if let Some((_, used_references)) = obj.extract_used_references() {
                for used_ref in used_references {
                    if !self
                        .internal_object_store
                        .contains_key(&used_ref.fully_qualified_name)
                        && !self
                            .external_object_store
                            .contains_key(&used_ref.fully_qualified_name)
                    {
                        let suggestion =
                            self.suggest_reference(&used_ref.fully_qualified_name, 5, 5);

                        if let Some(c) = suggestion {
                            errors.push(eyre!(
                                "unknown reference: {}, in object {} did you mean {}?",
                                used_ref.fully_qualified_name,
                                key,
                                c
                            ));
                        } else {
                            errors.push(eyre!(
                                "unknown reference: {}, in object {}",
                                used_ref.fully_qualified_name,
                                key,
                            ));
                        }
                    }
                }
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn suggest_reference(
        &self,
        unknown_reference: &str,
        max_length_distance: usize,
        max_edit_distance: usize,
    ) -> Option<String> {
        let best_internal_candidate = suggest_known_alternative(
            unknown_reference,
            self.internal_object_store.keys().cloned().collect(),
            max_length_distance,
            max_edit_distance,
        );
        let best_external_candidate = suggest_known_alternative(
            unknown_reference,
            self.external_object_store.keys().cloned().collect(),
            max_length_distance,
            max_edit_distance,
        );
        match (best_internal_candidate, best_external_candidate) {
            (None, None) => None,
            (None, Some((external, _score))) => Some(external.clone()),
            (Some((internal, _score)), None) => Some(internal.clone()),
            // very unlikely to happen, but just in case, we'll prefer
            // suggesting internal references
            (Some((internal, internal_score)), Some((external, external_score))) => {
                if external_score > internal_score {
                    Some(external.clone())
                } else {
                    Some(internal.clone())
                }
            }
        }
    }

    //TODO: This is not an efficient way to do this, but for the test cases it works,
    //at some point we should find a more high performance solution.
    // see: https://github.com/aslowwriter/snakedown/issues/55
    pub fn pre_process<R: Renderer>(&mut self, render: R, site_rel_api_path: &Path) -> Result<()> {
        for object in self.internal_object_store.values_mut() {
            if let Some((mut object_docstring, used_references)) = object.extract_used_references()
            {
                for used_ref in used_references {
                    let display_text = used_ref
                        .clone()
                        .display_text
                        .or_else(|| Some(used_ref.fully_qualified_name.clone()));

                    let target = self
                        .external_object_store
                        .get(&used_ref.fully_qualified_name)
                        .map(|u| u.as_str().to_string())
                        .unwrap_or_else(|| used_ref.fully_qualified_name.clone());

                    let expanded_ref =
                        render.render_reference(display_text, site_rel_api_path, target)?;
                    object_docstring =
                        object_docstring.replace(&used_ref.original(), &expanded_ref);
                }
                object.replace_docstring(Some(object_docstring));
            }
        }

        Ok(())
    }

    fn serialize_pages<R: Renderer>(
        &self,
        renderer: &R,
        ctx: &Context,
        out_api_path: &Path,
        skip_write: bool,
    ) -> Result<()> {
        if !skip_write {
            create_dir_all(out_api_path)?;
        }

        for (key, object) in self.internal_object_store.iter() {
            let file_path = out_api_path.join(key).with_added_extension("md");
            let rendered = render_object(object, key.clone(), &renderer, ctx)?;
            let rendered_trimmed = rendered.trim_start();
            if !skip_write {
                let mut file = File::create(file_path)?;
                file.write_all(rendered_trimmed.as_bytes())?;
            }
        }

        if let Some((index_file_path, index_file_content)) =
            &renderer.index_file(Some("API".to_string()))
            && !skip_write
        {
            let mut file = File::create(out_api_path.join(index_file_path))?;
            file.write_all(index_file_content.as_bytes())?;
        }
        Ok(())
    }

    fn serialize_notebooks<R: Renderer>(
        &self,
        notebook_out_path: &Path,
        renderer: &R,
        skip_write: bool,
    ) -> Result<()> {
        if !skip_write {
            create_dir_all(notebook_out_path)?;
        }
        for (key, cells) in self.notebook_store.iter() {
            let dir_path = notebook_out_path.join(key);
            let file_path = dir_path.clone().join("index").with_added_extension("md");
            let mut rendered = render_notebook(
                dir_path
                    .file_stem()
                    .map(|p| p.display().to_string())
                    .as_deref(),
                cells,
                &renderer,
            )?;
            // some tools insert an extra EOL at the end of the file
            if !rendered.text.ends_with("\n") {
                rendered.text.push('\n');
            }

            if !skip_write {
                create_dir_all(dir_path.clone())?;
                let mut file = File::create(file_path)?;
                file.write_all(rendered.text.as_bytes())?;
                for img in rendered.images {
                    let mut img_file = File::create(dir_path.join(img.name))?;
                    img_file.write_all(&img.data)?;
                }
            }
        }

        Ok(())
    }

    pub fn serialize<R: Renderer>(
        self,
        renderer: &R,
        ctx: &Context,
        out_api_path: &Path,
        notebook_out_path: Option<&Path>,
        skip_write: bool,
    ) -> Result<()> {
        self.serialize_pages(renderer, ctx, out_api_path, skip_write)?;
        if let Some(notebook_out_path) = &notebook_out_path {
            self.serialize_notebooks(notebook_out_path, renderer, skip_write)?;
        }
        Ok(())
    }
}

pub fn should_include_class(
    class_docs: &ClassDocumentation,
    skip_private: bool,
    skip_undoc: bool,
) -> bool {
    (!skip_undoc || class_docs.docstring.is_some())
        && !(skip_private && class_docs.name.starts_with("_"))
}

pub fn should_include_function(
    func_docs: &FunctionDocumentation,
    skip_private: bool,
    skip_undoc: bool,
) -> bool {
    (!skip_undoc || func_docs.docstring.is_some())
        && !(skip_private && func_docs.name.starts_with("_"))
}

pub fn should_include_module(mod_docs: &ModuleDocumentation, skip_undoc: bool) -> bool {
    !skip_undoc || mod_docs.docstring.is_some()
}

pub fn index_functions(
    index: &mut RawIndex,
    func_docs: &FunctionDocumentation,
    prefix: String,
) -> Result<()> {
    let full_prefix = format!("{}.{}", prefix, func_docs.name);
    tracing::debug!("Indexing {}", &full_prefix);

    // try_insert isn't stable yet
    #[allow(clippy::map_entry)]
    if index.internal_object_store.contains_key(&full_prefix) {
        Err(eyre!("tried to insert duplicate key: {}", &full_prefix))
    } else {
        index.internal_object_store.insert(
            full_prefix,
            ObjectDocumentation::Function(func_docs.clone()),
        );
        Ok(())
    }
}

pub fn index_class(
    index: &mut RawIndex,
    class_docs: &ClassDocumentation,
    prefix: String,
) -> Result<()> {
    let full_prefix = format!("{}.{}", prefix, class_docs.name);
    tracing::debug!("Indexing {}", &full_prefix);

    if index.internal_object_store.contains_key(&full_prefix) {
        Err(eyre!("tried to insert duplicate key: {}", &full_prefix))
    } else {
        for meth_doc in &class_docs.methods {
            index_functions(index, meth_doc, full_prefix.clone())?;
        }
        index
            .internal_object_store
            .insert(full_prefix, ObjectDocumentation::Class(class_docs.clone()));
        Ok(())
    }
}

/// from import as in `from a.b.c import d`
///                         -----
pub fn get_from_import_path(pkg_name: String, relative_module_file_path: &Path) -> Result<String> {
    let mut import_components = vec![pkg_name];
    let components: Vec<String> = relative_module_file_path
        .with_extension("")
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .map(String::from)
        .collect::<Vec<String>>();

    import_components.extend(components);

    Ok(import_components.join("."))
}

pub fn suggest_known_alternative(
    unknown_reference: &str,
    alternatives: Vec<String>,
    max_length_distance: usize,
    max_edit_distance: usize,
) -> Option<(String, usize)> {
    let candidate_length = &unknown_reference.chars().count();
    let mut candidates = alternatives
        .iter()
        .filter(|k| k.chars().count().abs_diff(*candidate_length) < max_length_distance)
        .map(|k| (k.to_string().clone(), edit_distance(k, unknown_reference)))
        .filter(|(_, score)| score < &max_edit_distance)
        .collect::<Vec<(String, usize)>>();

    candidates.sort_by_key(|a| a.1);

    candidates.first().cloned()
}

#[cfg(test)]
mod test {

    use super::suggest_known_alternative;
    use color_eyre::Result;

    #[test]
    fn suggest_alternatives_garbage() -> Result<()> {
        let known_keys: Vec<String> = vec![
            "test_pkg.bar.greet",
            "test_pkg.bar.Greeter",
            "test_pkg.bar.Greeter.greet",
            "numpy.fft",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let unknown_ref = "asdfasdfasdfasdfasdf";

        let suggested_ref = suggest_known_alternative(unknown_ref, known_keys, 5, 5);

        assert_eq!(suggested_ref, None);
        Ok(())
    }

    #[test]
    fn suggest_alternatives_external() -> Result<()> {
        let known_keys: Vec<String> = vec![
            "test_pkg.bar.greet",
            "test_pkg.bar.Greeter",
            "test_pkg.bar.Greeter.greet",
            "numpy.fft",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let unknown_ref = "nimpy.fft";

        let suggested_ref = suggest_known_alternative(unknown_ref, known_keys, 5, 5);

        assert_eq!(suggested_ref, Some(("numpy.fft".to_string(), 1)));
        Ok(())
    }
    #[test]
    fn suggest_alternatives_internal() -> Result<()> {
        let known_keys: Vec<String> = vec![
            "test_pkg.bar.greet",
            "test_pkg.bar.Greeter",
            "test_pkg.bar.Greeter.greet",
            "numpy.fft",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
        let unknown_ref = "test_pkg.bar.great";

        let suggested_ref = suggest_known_alternative(unknown_ref, known_keys, 5, 5);

        assert_eq!(suggested_ref, Some(("test_pkg.bar.greet".to_string(), 1)));
        Ok(())
    }
}
