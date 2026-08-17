use rustpython_parser::ast::{Identifier, StmtClassDef};

use crate::indexing::content::ContentNode;

use super::{function::FunctionDocumentation, utils::extract_docstring_from_body};

#[derive(Debug, Clone)]
pub struct ClassDocumentation {
    pub name: Identifier,
    pub docstring: Option<Vec<ContentNode>>,
    pub methods: Vec<FunctionDocumentation>,
}

impl ClassDocumentation {
    pub fn from_class_statements(value: &StmtClassDef, body_indent_level: usize) -> Self {
        Self {
            name: value.name.clone(),
            docstring: extract_docstring_from_body(&value.body, body_indent_level),
            methods: value
                .body
                .iter()
                .filter_map(|s| FunctionDocumentation::from_statements(s, body_indent_level))
                .collect(),
        }
    }
}

pub fn is_private_class(class_doc: &ClassDocumentation) -> bool {
    class_doc.name.starts_with("_")
}

#[cfg(test)]
mod test {

    use assert_fs::prelude::*;
    use color_eyre::Result;
    use std::{fs::File, io::Write};

    use crate::indexing::content::ContentNode;
    use crate::parsing::{
        python::module::extract_module_documentation,
        python::utils::{parse_python_file, parse_python_str},
    };

    fn test_python_class() -> &'static str {
        r#"
class Greeter:
    '''
    this is a class docstring.

        this line has exactly one indent!



    '''

    class_var = "whatever"

    def greet(self):
        print("Hello, world!")
        def inner():
            print("this is a closure!")
        inner()
    "#
    }
    #[test]
    fn parse_test_python_class() -> Result<()> {
        let program = parse_python_str(test_python_class())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 0);
        assert_eq!(documentation.classes.len(), 1);

        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let class = documentation.classes.first().unwrap();

        assert_eq!(class.methods.len(), 1);

        Ok(())
    }
    #[test]
    fn parse_test_python_class_docstring() -> Result<()> {
        let program = parse_python_str(test_python_class())?;

        let documentation = extract_module_documentation(&program, false, false);

        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let docstring = documentation.classes.first().unwrap().docstring.clone();

        assert_eq!(
            docstring,
            Some(vec![ContentNode::Text(String::from(
                r"
this is a class docstring.

    this line has exactly one indent!



"
            ))])
        );
        Ok(())
    }
    #[test]
    fn parse_test_python_file_on_disk() -> Result<()> {
        let file_contents = test_python_class();

        let temp_dir = assert_fs::TempDir::new()?;
        let child = temp_dir.child("foo.py");
        child.touch()?;
        let root_pkg_path = child.path();
        let mut file = File::create(root_pkg_path)?;
        file.write_all(file_contents.as_bytes())?;

        let program = parse_python_file(root_pkg_path)?;
        let docs = extract_module_documentation(&program, false, false);

        assert_eq!(docs.docstring, None);
        assert_eq!(docs.functions.len(), 0);
        assert_eq!(docs.classes.len(), 1);

        Ok(())
    }
}
