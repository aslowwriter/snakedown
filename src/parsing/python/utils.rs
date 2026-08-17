use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use rustpython_parser::{
    Mode,
    ast::{Constant, Expr, ExprConstant, Mod, Stmt, StmtExpr},
    parse,
};
use std::{fs::File, io::Read, path::Path};

use crate::indexing::content::{ContentNode, parse_contents};

pub fn parse_python_file(path: &Path) -> Result<Mod> {
    let mut file = File::open(path)?;
    let mut file_content = String::new();
    file.read_to_string(&mut file_content)?;
    let program = parse_python_str(&file_content)?;
    Ok(program)
}

pub fn parse_python_str(content: &str) -> Result<Mod> {
    let parsed = parse(content, Mode::Module, "<embedded>");
    Ok(parsed?)
}

pub(crate) fn extract_docstring_from_body(
    body: &[Stmt],
    indent_level: usize,
) -> Option<Vec<ContentNode>> {
    match body.first() {
        Some(Stmt::Expr(StmtExpr { range: _, value })) => {
            if let Expr::Constant(ExprConstant {
                range: _,
                value: Constant::Str(s),
                kind: _,
            }) = &**value
            {
                let raw_docstring = s;

                // API specifies that we might fail
                // but this should never happen for wellformed python
                // since that would have been an indent error earlier
                // so if we touch that case, we did something wrong
                #[allow(clippy::expect_used)]
                let docstring_stripped: String = raw_docstring
                    .lines()
                    .map(|line| {
                        // PEP 8 says use 4 spaces, and that is mostly the default
                        // might make this more flexible at some point
                        let prefix = "    ".repeat(indent_level);

                        if line.is_empty()
                            || !line
                                .chars()
                                .next()
                                .map(|c| c.is_whitespace())
                                .unwrap_or(false)
                        {
                            // let's not touch empty lines
                            Ok(line)
                        } else {
                            line.strip_prefix(&prefix).ok_or_eyre(eyre!(
                                "line `{}` did not contain prefix {:?} (expected indent level: {})",
                                line,
                                prefix,
                                indent_level
                            ))
                        }
                    })
                    .collect::<Result<Vec<&str>>>()
                    .expect("Could not consistently strip prefix. most likely a bug in program")
                    .join("\n");
                parse_contents(&mut docstring_stripped.as_ref()).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod test {

    use crate::parsing::python::module::extract_module_documentation;

    use super::*;

    #[test]
    fn parse_empty_string() -> Result<()> {
        let program = parse_python_str("")?;
        let documentation = extract_module_documentation(&program, false, false);

        assert_eq!(documentation.docstring, None);
        assert_eq!(documentation.functions.len(), 0);
        assert_eq!(documentation.classes.len(), 0);

        Ok(())
    }
}
