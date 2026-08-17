use std::path::PathBuf;

use color_eyre::{Result, eyre::eyre};
use rustpython_parser::ast::{Mod, Stmt, StmtAssign};

use crate::indexing::content::ContentNode;

use super::{
    class::{ClassDocumentation, is_private_class},
    function::{FunctionDocumentation, is_private_function},
    utils::extract_docstring_from_body,
};

#[derive(Default, Debug, Clone)]
pub struct ModuleDocumentation {
    pub docstring: Option<Vec<ContentNode>>,
    pub functions: Vec<FunctionDocumentation>,
    pub classes: Vec<ClassDocumentation>,
    pub sub_modules: Option<Vec<PathBuf>>,
    pub exports: Option<Vec<String>>,
}

// just a conveneience function so we don't have to worry about
// inline modules defined in interactive sessions that we
// don't have to handle here but which are technically possible
pub fn extract_module_documentation(
    input_module: &Mod,
    skip_private: bool,
    skip_undoc: bool,
) -> ModuleDocumentation {
    if let Mod::Module(mod_module) = input_module {
        // a module is required to have indent 0
        extract_documentation_from_statements(&mod_module.body, skip_private, skip_undoc)
    } else {
        ModuleDocumentation::default()
    }
}

fn extract_exports_from_statement(statement: &StmtAssign) -> Result<Vec<String>> {
    if !statement
        .clone()
        .targets
        .into_iter()
        .filter_map(|t| t.name_expr())
        .any(|e| e.id == *"__all__")
    {
        return Err(eyre!("target of assignment was not __all__"));
    };
    match &*statement.value.clone() {
        rustpython_parser::ast::Expr::List(expr_list) => Ok(expr_list
            .elts
            .iter()
            .filter_map(|e| e.as_constant_expr())
            .filter_map(|c| c.value.as_str())
            .cloned()
            .collect::<Vec<String>>()),
        _ => Err(eyre!("__all__ assignment was not list")),
    }
}

fn extract_documentation_from_statements(
    statements: &[Stmt],
    skip_private: bool,
    skip_undoc: bool,
) -> ModuleDocumentation {
    let mut free_functions = vec![];
    let mut class_definitions = vec![];
    let mut exports = None;
    // a module is required to have indent 0
    let docstring = extract_docstring_from_body(statements, 0);
    for statement in statements {
        if let Stmt::Assign(stmt_assign) = statement {
            match (&mut exports, extract_exports_from_statement(stmt_assign)) {
                (None, Ok(exported)) => exports = Some(exported),
                (Some(_), Ok(new_exported)) => {
                    tracing::warn!("__all__ was defined multiple times.");
                    exports = Some(new_exported);
                }
                _ => (),
            }
        }
        if let Stmt::FunctionDef(stmt_function_def) = statement {
            let function_doc: FunctionDocumentation =
                FunctionDocumentation::from_function_statements(stmt_function_def, 1);
            if function_doc.docstring.is_none() && skip_undoc {
                tracing::debug!(
                    "skipping function {} because it is undocumented",
                    function_doc.name,
                );
                continue;
            };

            if is_private_function(&function_doc) && skip_private {
                tracing::debug!(
                    "skipping function {} because it is private",
                    function_doc.name,
                );
                continue;
            }
            free_functions.push(function_doc);
        }
        if let Stmt::AsyncFunctionDef(stmt_async_function_def) = statement {
            let function_doc: FunctionDocumentation =
                FunctionDocumentation::from_async_function_statements(stmt_async_function_def, 1);
            if function_doc.docstring.is_none() && skip_undoc {
                tracing::debug!(
                    "skipping function {} because it is undocumented",
                    function_doc.name,
                );
                continue;
            };

            if is_private_function(&function_doc) && skip_private {
                tracing::debug!(
                    "skipping function {} because it is private",
                    function_doc.name,
                );
                continue;
            }
            free_functions.push(function_doc);
        }
        if let Stmt::ClassDef(stmt_class_def) = statement {
            let class_doc: ClassDocumentation =
                ClassDocumentation::from_class_statements(stmt_class_def, 1);
            if is_private_class(&class_doc) && skip_private {
                tracing::debug!("skipping class {} because it is private", class_doc.name,);
                continue;
            }
            if class_doc.docstring.is_none() && skip_undoc {
                tracing::debug!(
                    "skipping function {} because it is undocumented",
                    class_doc.name,
                );
                continue;
            };
            class_definitions.push(class_doc);
        }
    }

    ModuleDocumentation {
        docstring,
        functions: free_functions,
        classes: class_definitions,
        sub_modules: None,
        exports,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use color_eyre::Result;
    use rustpython_parser::{Mode, parse};
    use tracing_test::traced_test;

    #[test]
    fn test_doc_extraction_interactive_module() -> Result<()> {
        let expr = parse("1 + 2", Mode::Expression, "<embedded>")?;
        let docs = extract_module_documentation(&expr, false, false);

        assert_eq!(docs.docstring, None);
        assert_eq!(docs.functions.len(), 0);
        assert_eq!(docs.classes.len(), 0);

        Ok(())
    }
    #[test]
    fn test_doc_extraction_skip_undoc_and_private_module() -> Result<()> {
        let expr = parse(
            r#"
def foo():
    """asdf"""
    pass

def _bar():
    """asdf"""
    pass

def baz():
    pass

class Cls:
    """normal class"""


class _Cls:
    """normal class"""

class UndocClass:
    pass
"#,
            Mode::Module,
            "<embedded>",
        )?;
        let docs = extract_module_documentation(&expr, true, true);

        assert_eq!(docs.docstring, None);
        assert_eq!(docs.functions.len(), 1);
        assert_eq!(docs.classes.len(), 1);

        Ok(())
    }

    #[test]
    fn test_doc_extraction_exports() -> Result<()> {
        let expr = parse(
            r#"

__all__ = ["a", "b", "c", "d", "foo", 4 , 5]

a = 1
b = 3
c,d, foo = *bar
"#,
            Mode::Module,
            "<embedded>",
        )?;
        let docs = extract_module_documentation(&expr, true, true);

        assert_eq!(docs.exports.map(|e| e.len()), Some(5));

        Ok(())
    }
    #[test]
    #[traced_test]
    fn test_doc_extraction_multiple_exports() -> Result<()> {
        let expr = parse(
            r#"

__all__ = ["a"]
__all__ = ["b"]

a = 1
b = 3
"#,
            Mode::Module,
            "<embedded>",
        )?;
        let docs = extract_module_documentation(&expr, true, true);

        assert_eq!(docs.exports, Some(vec![String::from("b")]));
        assert!(logs_contain("__all__ was defined multiple times."));

        Ok(())
    }
    #[test]
    fn test_doc_extraction_export_non_list() -> Result<()> {
        let expr = parse(
            r#"

__all__ = "a"

a = 1
b = 3
"#,
            Mode::Module,
            "<embedded>",
        )?;
        let docs = extract_module_documentation(&expr, true, true);

        assert_eq!(docs.exports, None);

        Ok(())
    }
}
