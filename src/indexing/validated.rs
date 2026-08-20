use crate::indexing::raw::suggest_known_alternative;
use crate::indexing::serializable::SerializableIndex;
use crate::indexing::{content::ContentNode, raw::RawIndex};
use crate::parsing::ObjectDocumentation;
use crate::parsing::python::class::ValidatedClassDocumentation;
use crate::parsing::python::function::ValidatedFunctionDocumentation;
use crate::parsing::python::module::ValidatedModuleDocumentation;
use crate::parsing::python::object::ValidatedObjectDocumentation;
use crate::render::formats::Renderer;
use nbformat::v4::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tera::Context;

pub struct ValidatedIndex {
    pub pkg_name: String,
    pub pkg_root: PathBuf,
    pub page_store: HashMap<String, Page>,
    pub notebook_store: HashMap<String, Vec<Cell>>,
}

#[derive(Debug)]
pub struct Page {
    pub fully_qualified_name: String,
    pub content: ValidatedObjectDocumentation,
}

pub struct NotebookPage {
    pub fully_qualified_name: String,
    pub content: Vec<ValidatedContentNode>,
}

#[derive(Debug, Clone)]
pub struct ValidReference {
    pub target: String,
    pub display: String,
}

#[derive(Debug, Clone)]
pub struct InvalidReference {
    pub org: String,
    pub suggestions: Option<String>,
    pub source: String,
}

/// The reason that validated content nodes can be an invalid reference
/// is that we may want to keep going based on permissiveness, so this way we still know which
/// references are valid and which aren't
#[derive(Debug, Clone)]
pub enum ValidatedContentNode {
    Text(String),
    ValidReference(ValidReference),
    InvalidReference(InvalidReference),
}

impl ValidatedIndex {
    pub fn from_raw(raw: RawIndex) -> (Self, HashMap<String, Vec<InvalidReference>>) {
        let mut page_store = HashMap::new();
        let mut invalid_references = HashMap::new();

        tracing::info!("validating references in python objects.");
        for (key, obj) in raw.internal_object_store.iter() {
            tracing::debug!("validating references in {}.", key);
            let validated_content = match obj.docstring() {
                None => Vec::new(),
                Some(docstring) => docstring
                    .iter()
                    .map(|node| match node {
                        ContentNode::Text(t) => ValidatedContentNode::Text(t.to_string()),
                        ContentNode::Reference(used_ref) => {
                            // get tdisplay text, falling back to the fqn if necessary
                            let display_text = used_ref
                                .clone()
                                .display_text
                                .unwrap_or_else(|| used_ref.fully_qualified_name.clone());

                            // try to get a target from the external or internal object store
                            let t = raw
                                .external_object_store
                                .get(&used_ref.fully_qualified_name)
                                .map(|u| u.as_str().to_string())
                                .or_else(|| {
                                    if raw
                                        .internal_object_store
                                        .contains_key(&used_ref.fully_qualified_name)
                                    {
                                        Some(used_ref.fully_qualified_name.clone())
                                    } else {
                                        None
                                    }
                                });

                            // if we found one, we're good
                            if let Some(target) = t {
                                ValidatedContentNode::ValidReference(ValidReference {
                                    target,
                                    display: display_text,
                                })
                            } else {
                                // if we didn't, that's an invalid reference
                                tracing::warn!(
                                    "Found invalid reference {}.",
                                    &used_ref.fully_qualified_name
                                );
                                let suggestion =
                                    suggest_reference(&raw, &used_ref.fully_qualified_name, 5, 5);
                                let invalid_ref = InvalidReference {
                                    org: used_ref.fully_qualified_name.clone(),
                                    suggestions: suggestion,
                                    source: String::from("foo"),
                                };
                                let v = invalid_references
                                    .entry(key.clone())
                                    .or_insert_with(Vec::new);
                                v.push(invalid_ref.clone());
                                ValidatedContentNode::InvalidReference(invalid_ref)
                            }
                        }
                    })
                    .collect(),
            };
            // the rest is a fairly simple conversion as we only just extract name
            // information
            // from here everything we need should be in the object store so we can
            // always just look it up. The exception being things like function return
            // types, which we'll need to keep around
            let validated_docs = match obj {
                ObjectDocumentation::Module(module) => {
                    ValidatedObjectDocumentation::Module(ValidatedModuleDocumentation {
                        docstring: Some(validated_content),
                        functions: module
                            .functions
                            .iter()
                            .map(|f| f.name.to_string())
                            .collect(),
                        classes: module
                            .classes
                            .iter()
                            .map(|cls| cls.name.to_string())
                            .collect(),
                        sub_modules: module.sub_modules.clone().map(|sub_modules| {
                            sub_modules
                                .iter()
                                .map(|sm| sm.display().to_string())
                                .collect()
                        }),
                        exports: module.exports.clone(),
                    })
                }
                ObjectDocumentation::Class(class) => {
                    ValidatedObjectDocumentation::Class(ValidatedClassDocumentation {
                        name: class.name.to_string(),
                        docstring: Some(validated_content),
                        method_names: class.methods.iter().map(|f| f.name.clone()).collect(),
                    })
                }
                ObjectDocumentation::Function(function) => {
                    ValidatedObjectDocumentation::Function(ValidatedFunctionDocumentation {
                        name: function.name.clone(),
                        docstring: Some(validated_content),
                        return_type: function.return_type.clone(),
                        args: function.args.clone(),
                        generics: function.generics.clone(),
                    })
                }
            };
            let page = Page {
                fully_qualified_name: key.to_string(),
                content: validated_docs,
            };
            page_store.insert(key.to_string(), page);
        }

        (
            ValidatedIndex {
                page_store,
                notebook_store: raw.notebook_store,
                pkg_name: raw.pkg_name,
                pkg_root: raw.pkg_root,
            },
            invalid_references,
        )
    }
    pub fn render<R: Renderer>(
        self,
        renderer: &R,
        ctx: &Context,
        notebook_out_path: Option<&Path>,
    ) -> SerializableIndex {
        SerializableIndex::from_validated(self, renderer, ctx, notebook_out_path)
    }
}

fn suggest_reference(
    raw: &RawIndex,
    unknown_reference: &str,
    max_length_distance: usize,
    max_edit_distance: usize,
) -> Option<String> {
    let best_internal_candidate = suggest_known_alternative(
        unknown_reference,
        raw.internal_object_store.keys().cloned().collect(),
        max_length_distance,
        max_edit_distance,
    );
    let best_external_candidate = suggest_known_alternative(
        unknown_reference,
        raw.external_object_store.keys().cloned().collect(),
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
