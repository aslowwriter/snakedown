pub mod args;
pub mod expr;
pub mod formats;
pub mod jupyter;
use color_eyre::Result;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use strum::Display;
use tera::{Context, Tera};

use args::render_args;
use expr::render_expr;

use crate::{
    indexing::content::ContentNode,
    parsing::{
        ObjectDocumentation,
        python::{
            class::ClassDocumentation, function::FunctionDocumentation, module::ModuleDocumentation,
        },
    },
    render::formats::Renderer,
};

#[derive(Clone, Copy, Debug, Display, ValueEnum, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub enum SSG {
    Markdown,
    Zola,
}

pub fn translate_filename(path: &Path) -> PathBuf {
    let mut translated = path.with_extension("md");
    if translated.file_stem() == Some(OsStr::new("__init__")) {
        translated = translated.with_file_name("_index.md");
    }

    translated
}

pub fn fully_qualified_object_name(object: &ObjectDocumentation, prefix: Option<String>) -> String {
    match (object, prefix) {
        (ObjectDocumentation::Module(_), None) => String::new(),
        (ObjectDocumentation::Module(_), Some(p)) => p,
        (ObjectDocumentation::Class(class_documentation), None) => {
            class_documentation.name.to_string().trim().to_string()
        }
        (ObjectDocumentation::Class(class_documentation), Some(p)) => {
            format!("{}.{}", p, class_documentation.name.clone().trim())
        }
        (ObjectDocumentation::Function(function_documentation), None) => {
            function_documentation.name.to_string().trim().to_string()
        }
        (ObjectDocumentation::Function(function_documentation), Some(p)) => {
            format!("{}.{}", p, function_documentation.name.to_string().trim())
        }
    }
}

pub fn render_object<R: Renderer>(
    object: &ObjectDocumentation,
    fully_qualified_name: String,
    renderer: &R,
    ctx: &Context,
) -> Result<String> {
    match object {
        ObjectDocumentation::Class(class_documentation) => Ok(render_class_docs(
            class_documentation,
            &fully_qualified_name,
            renderer,
            ctx,
        )?),
        ObjectDocumentation::Module(module_documentation) => Ok(render_module(
            module_documentation,
            fully_qualified_name,
            renderer,
            ctx,
        )?),
        ObjectDocumentation::Function(function_documentation) => Ok(render_function_docs(
            function_documentation,
            &fully_qualified_name,
            renderer,
            ctx,
        )?),
    }
}

pub fn render_module<R: Renderer>(
    mod_doc: &ModuleDocumentation,
    fully_qualified_name: String,
    renderer: &R,
    ctx: &Context,
) -> Result<String> {
    let mut local_ctx = ctx.clone();

    let front_matter = &renderer.render_front_matter(Some(&fully_qualified_name));
    local_ctx.insert("SNAKEDOWN_FRONT_MATTER", &front_matter);

    if let Some(docstring_nodes) = mod_doc.docstring.clone() {
        let docstring = docstring_nodes
            .into_iter()
            .map(|node: ContentNode| match node {
                ContentNode::Text(t) => t,
                ContentNode::Reference(user_reference) => renderer.render_reference(
                    user_reference.fully_qualified_name,
                    user_reference.display_text,
                ),
            })
            .collect::<Vec<_>>()
            .join("");
        local_ctx.insert("SNAKEDOWN_MODULE_DOCSTRING", docstring.trim());
    }

    let function_template = r#"{{ SNAKEDOWN_FRONT_MATTER }}
{%if SNAKEDOWN_MODULE_DOCSTRING%}
{{SNAKEDOWN_MODULE_DOCSTRING}}
{%endif%}"#;

    Ok(Tera::one_off(function_template, &local_ctx, false)?)
}

fn render_class_docs<R: Renderer>(
    class_docs: &ClassDocumentation,
    fully_qualified_name: &str,
    renderer: &R,
    ctx: &Context,
) -> Result<String> {
    let mut local_ctx = ctx.clone();

    let front_matter = &renderer.render_front_matter(Some(fully_qualified_name));
    local_ctx.insert("SNAKEDOWN_FRONT_MATTER", &front_matter);

    if let Some(docstring_nodes) = class_docs.docstring.clone() {
        let docstring = docstring_nodes
            .into_iter()
            .map(|node: ContentNode| match node {
                ContentNode::Text(t) => t,
                ContentNode::Reference(user_reference) => renderer.render_reference(
                    user_reference.fully_qualified_name,
                    user_reference.display_text,
                ),
            })
            .collect::<Vec<_>>()
            .join("");
        local_ctx.insert("SNAKEDOWN_CLASS_DOCSTRING", docstring.trim());
    }

    let function_template = r#"
        {{ SNAKEDOWN_FRONT_MATTER }}
{%if SNAKEDOWN_CLASS_DOCSTRING%}
{{SNAKEDOWN_CLASS_DOCSTRING}}
{%endif%}"#;

    Ok(Tera::one_off(function_template, &local_ctx, false)?)
}

fn render_function_docs<R: Renderer>(
    fn_docs: &FunctionDocumentation,
    fully_qualified_name: &str,
    renderer: &R,
    ctx: &Context,
) -> Result<String> {
    let mut local_ctx = ctx.clone();

    let front_matter = &renderer.render_front_matter(Some(fully_qualified_name));
    local_ctx.insert("SNAKEDOWN_FRONT_MATTER", &front_matter);
    local_ctx.insert("SNAKEDOWN_FUNCTION_NAME", &fn_docs.name);
    local_ctx.insert(
        "SNAKEDOWN_FUNCTION_ARGS",
        &render_args(fn_docs.args.clone()),
    );
    if let Some(ret) = fn_docs.return_type.clone() {
        local_ctx.insert("SNAKEDOWN_FUNCTION_RET", &render_expr(ret));
    }

    if let Some(docstring_nodes) = fn_docs.docstring.clone() {
        let docstring = docstring_nodes
            .into_iter()
            .map(|node: ContentNode| match node {
                ContentNode::Text(t) => t,
                ContentNode::Reference(user_reference) => renderer.render_reference(
                    user_reference.fully_qualified_name,
                    user_reference.display_text,
                ),
            })
            .collect::<Vec<_>>()
            .join("");
        local_ctx.insert("SNAKEDOWN_FUNCTION_DOCSTRING", docstring.trim());
    }

    let function_template = r#"
        {{ SNAKEDOWN_FRONT_MATTER }}

{{ SNAKEDOWN_FUNCTION_NAME }}({{ SNAKEDOWN_FUNCTION_ARGS }}){% if SNAKEDOWN_FUNCTION_RET %} -> {{ SNAKEDOWN_FUNCTION_RET }}{%endif%}
{%if SNAKEDOWN_FUNCTION_DOCSTRING%}
{{SNAKEDOWN_FUNCTION_DOCSTRING}}
{%endif%}"#;

    Ok(Tera::one_off(function_template, &local_ctx, false)?)
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use tera::Context;

    use crate::{
        parsing::{python::module::extract_module_documentation, python::utils::parse_python_str},
        render::{
            formats::{md::MdRenderer, zola::ZolaRenderer},
            render_module, translate_filename,
        },
    };
    fn test_dirty_module_str() -> &'static str {
        r"'''This is a module that is used to test snakedown.'''

from typing import Any

__all__ = ['foo']

def foo(bar: int) -> Dict[str, Any]:
    '''this is a docstring for the foo function'''

    bar += 15
    bar << bar | 19
    return 0

class Greeter:
    '''
    this is a class docstring.

    '''

    class_var = 'whatever'

    def greet(self, name, *args, foo: str = 'bar', **kwargs) -> Callable[[], None]:
        '''





        Greet the world.

        Parameters
        ----------
        name: str
            just a parameter. it's actually used for anything

        Returns
        -------
        Callable[[], None]
            just a random closure to make the types interesting to render.




        '''
        print('Hello, world!')
        def inner():
            print('this is a closure!')
        inner()
        "
    }

    fn expected_module_docs_rendered() -> &'static str {
        r#"# snakedown.testing.test_module

This is a module that is used to test snakedown.
"#
    }

    #[test]
    fn render_module_documentation() -> Result<()> {
        let parsed = parse_python_str(test_dirty_module_str())?;
        let mod_documentation = extract_module_documentation(&parsed, false, false);
        let ctx = Context::new();

        let rendered = render_module(
            &mod_documentation,
            String::from("snakedown.testing.test_module"),
            &MdRenderer::new(),
            &ctx,
        )?;

        assert_eq!(rendered, expected_module_docs_rendered());

        Ok(())
    }

    fn expected_module_docs_zola_rendered() -> &'static str {
        r#"+++
title = "snakedown"
+++

This is a module that is used to test snakedown.
"#
    }

    #[test]
    fn render_module_documentation_zola() -> Result<()> {
        let parsed = parse_python_str(test_dirty_module_str())?;
        let mod_documentation = extract_module_documentation(&parsed, false, false);
        let ctx = Context::new();

        let rendered = render_module(
            &mod_documentation,
            String::from("snakedown"),
            &ZolaRenderer::default(),
            &ctx,
        )?;

        assert_eq!(rendered, expected_module_docs_zola_rendered());

        Ok(())
    }
    #[test]
    fn test_translate_filename_init() -> Result<()> {
        let input = PathBuf::from("foo/bar/__init__.py");
        let expected = PathBuf::from("foo/bar/_index.md");
        assert_eq!(translate_filename(&input), expected);
        Ok(())
    }
    #[test]
    fn test_translate_filename_module() -> Result<()> {
        let input = PathBuf::from("foo/bar/baz.py");
        let expected = PathBuf::from("foo/bar/baz.md");
        assert_eq!(translate_filename(&input), expected);
        Ok(())
    }
}
