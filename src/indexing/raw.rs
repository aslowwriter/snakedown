use crate::parsing::python::utils::parse_python_str;
use crate::{
    config::ExternalIndex,
    indexing::validated::{InvalidReference, ValidatedIndex},
    parsing::{
        ObjectDocumentation,
        python::{
            class::ClassDocumentation,
            function::FunctionDocumentation,
            jupyter::parse_notebook_file,
            module::{ModuleDocumentation, extract_module_documentation},
        },
    },
    should_include_reference,
};
use color_eyre::{Result, eyre::eyre};
use edit_distance::edit_distance;
use nbformat::v4::Cell;
use rustpython_parser::source_code::RandomLocator;
use sphinx_inv::SphinxInventoryReader;
use std::{collections::HashMap, path::PathBuf};
use std::{fs::File, io::Read, path::Path};
use tracing::{info, warn};
use url::Url;

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

        let mut file = File::open(&path)?;
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;
        let parsed = parse_python_str(&file_content);
        let mut locator = RandomLocator::new(&file_content);

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
                let mod_docs = extract_module_documentation(
                    &contents,
                    self.skip_private,
                    self.skip_undoc,
                    &mut locator,
                    PathBuf::from(self.pkg_name.clone()).join(rel_module_file_path),
                );
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

    pub fn validate(self) -> (ValidatedIndex, HashMap<String, Vec<InvalidReference>>) {
        info!("validating references");
        ValidatedIndex::from_raw(self)
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
