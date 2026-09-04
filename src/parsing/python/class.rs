use rustpython_parser::{
    ast::{Identifier, StmtClassDef},
    source_code::{OneIndexed, RandomLocator},
};
use std::path::PathBuf;

use crate::indexing::{content::ContentNode, validated::ValidatedContentNode};

use super::{function::FunctionDocumentation, utils::extract_docstring_from_body};

#[derive(Debug, Clone)]
pub struct ClassDocumentation {
    pub name: Identifier,
    pub docstring: Option<Vec<ContentNode>>,
    pub methods: Vec<FunctionDocumentation>,
    pub first_line: OneIndexed,
    pub last_line: OneIndexed,
    pub source_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidatedClassDocumentation {
    pub name: String,
    pub docstring: Option<Vec<ValidatedContentNode>>,
    pub method_names: Vec<String>,
    pub first_line: OneIndexed,
    pub last_line: OneIndexed,
    pub source_file: PathBuf,
}

impl ClassDocumentation {
    pub fn from_class_statements(
        value: &StmtClassDef,
        body_indent_level: usize,
        locator: &mut RandomLocator,
        source_file_path: PathBuf,
    ) -> Self {
        let first_line = locator.locate(value.range.start()).row;
        let last_line = locator.locate(value.range.end()).row;
        Self {
            name: value.name.clone(),
            docstring: extract_docstring_from_body(&value.body, body_indent_level),
            methods: value
                .body
                .iter()
                .filter_map(|s| {
                    FunctionDocumentation::from_statements(
                        s,
                        body_indent_level,
                        locator,
                        source_file_path.clone(),
                    )
                })
                .collect(),
            first_line,
            last_line,
            source_file: source_file_path,
        }
    }
}

pub fn is_private_class(class_doc: &ClassDocumentation) -> bool {
    class_doc.name.starts_with("_")
}

#[cfg(test)]
mod test {

    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use rustpython_parser::source_code::{OneIndexed, RandomLocator};
    use std::path::PathBuf;

    use crate::indexing::content::ContentNode;
    use crate::parsing::{
        python::module::extract_module_documentation, python::utils::parse_python_str,
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
        let content = test_python_class();
        let program = parse_python_str(content)?;
        let mut locator = RandomLocator::new(content);
        let documentation =
            extract_module_documentation(&program, false, false, &mut locator, PathBuf::new());
        assert_eq!(documentation.functions.len(), 0);
        assert_eq!(documentation.classes.len(), 1);

        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let class = documentation.classes.first().unwrap();

        assert_eq!(class.first_line, OneIndexed::from_zero_indexed(1));

        assert_eq!(class.methods.len(), 1);

        #[allow(clippy::unwrap_used)]
        let method = class.methods.first().unwrap();

        assert_eq!(method.first_line, OneIndexed::from_zero_indexed(13));

        Ok(())
    }
    #[test]
    fn parse_test_python_class_docstring() -> Result<()> {
        let program = parse_python_str(test_python_class())?;
        let mut locator = RandomLocator::new(test_python_class());

        let documentation =
            extract_module_documentation(&program, false, false, &mut locator, PathBuf::new());

        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let class = documentation.classes.first().unwrap();

        let docstring = class.docstring.clone();
        assert_eq!(class.first_line, OneIndexed::from_zero_indexed(1));

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
}
