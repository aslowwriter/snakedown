use rustpython_parser::ast::{
    Arguments, Expr, Stmt, StmtAsyncFunctionDef, StmtFunctionDef, TypeParam,
};

use crate::indexing::content::ContentNode;

use super::utils::extract_docstring_from_body;

#[derive(Debug, Clone)]
pub struct FunctionDocumentation {
    pub name: String,
    pub docstring: Option<Vec<ContentNode>>,
    pub return_type: Option<Expr>,
    pub args: Arguments,
    pub generics: Vec<TypeParam>,
}

impl FunctionDocumentation {
    pub fn from_statements(value: &Stmt, body_indent_level: usize) -> Option<Self> {
        match value {
            Stmt::AsyncFunctionDef(stmt_async_function_def) => {
                Some(FunctionDocumentation::from_async_function_statements(
                    stmt_async_function_def,
                    body_indent_level + 1,
                ))
            }
            Stmt::FunctionDef(stmt_function_def) => {
                Some(FunctionDocumentation::from_function_statements(
                    stmt_function_def,
                    body_indent_level + 1,
                ))
            }
            _ => None,
        }
    }
    pub fn from_async_function_statements(
        value: &StmtAsyncFunctionDef,
        body_indent_level: usize,
    ) -> Self {
        Self {
            name: value.name.to_string(),
            docstring: extract_docstring_from_body(&value.body, body_indent_level),
            return_type: value.returns.as_ref().map(|r| *r.clone()),
            args: *value.args.clone(),
            generics: value.type_params.clone(),
        }
    }
    pub fn from_function_statements(value: &StmtFunctionDef, body_indent_level: usize) -> Self {
        Self {
            name: value.name.to_string(),
            docstring: extract_docstring_from_body(&value.body, body_indent_level),
            return_type: value.returns.as_ref().map(|r| *r.clone()),
            args: *value.args.clone(),
            generics: value.type_params.clone(),
        }
    }
}

pub fn is_private_function(fn_doc: &FunctionDocumentation) -> bool {
    fn_doc.name.starts_with("_")
}

#[cfg(test)]
mod test {

    use crate::parsing::python::function::ContentNode;
    use color_eyre::Result;

    use crate::parsing::{
        python::module::extract_module_documentation, python::utils::parse_python_str,
    };

    fn test_python_func_no_types() -> &'static str {
        "
def is_odd(i):
    return bool(i & 1)
        "
    }
    fn test_python_async_func_no_types() -> &'static str {
        "async def is_odd(i):
    return bool(i & 1)
        "
    }

    fn test_python_async_func_docstring() -> &'static str {
        "async def is_odd(i):
    '''
    Determine whether a number is odd.

    Returns
    -------
        bool: True iff input number is odd
    '''
    return bool(i & 1)
        "
    }
    fn test_python_func_docstring() -> &'static str {
        "def is_odd(i):
    '''
    Determine whether a number is odd.

    Returns
    -------
        bool: True iff input number is odd
    '''
    return bool(i & 1)
        "
    }

    fn test_python_lambda() -> &'static str {
        "
def is_odd(i):
    inner = lambda x: x % 2
    return inner(i)
        "
    }
    fn test_python_closure() -> &'static str {
        "
def is_odd(i):
    def inner_func(i: float) -> bool:
        return bool(i&0)

    return not inner_func(i)
        "
    }
    fn test_python_no_func() -> &'static str {
        "
# this is a comment
a = 4
b = a + 6
assert b > 0
f = lambda a,b: [*a, *b]
        "
    }
    fn test_python_func_with_types() -> &'static str {
        "
def is_even(i: int) -> bool:
    return bool(i & 1)

        "
    }
    fn test_python_func_annotated() -> &'static str {
        "
def return_none(foo: str, bar, *args, unused: Dict[Any, str] = None) -> 4+9:
    return 22/7
        "
    }

    #[test]
    fn parse_doesnt_extract_lambda() -> Result<()> {
        let program = parse_python_str(test_python_lambda())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_async_func() -> Result<()> {
        let program = parse_python_str(test_python_async_func_no_types())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_doesnt_extract_closure() -> Result<()> {
        let program = parse_python_str(test_python_closure())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_func_no_types() -> Result<()> {
        let program = parse_python_str(test_python_func_no_types())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_no_func() -> Result<()> {
        let program = parse_python_str(test_python_no_func())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 0);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_func_dict_type() -> Result<()> {
        let program = parse_python_str(test_python_func_annotated())?;

        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_func_with_types() -> Result<()> {
        let program = parse_python_str(test_python_func_with_types())?;

        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_func_docstring() -> Result<()> {
        let program = parse_python_str(test_python_func_docstring())?;
        let documentation = extract_module_documentation(&program, false, false);
        assert_eq!(documentation.functions.len(), 1);
        assert_eq!(documentation.classes.len(), 0);
        Ok(())
    }
    #[test]
    fn parse_test_python_async_function_docstring() -> Result<()> {
        let program = parse_python_str(test_python_async_func_docstring())?;

        let documentation = extract_module_documentation(&program, false, false);
        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let function = documentation.functions.first().unwrap();
        let docstring = function.docstring.clone();
        assert_eq!(
            docstring,
            Some(vec![ContentNode::Text(String::from(
                r"
Determine whether a number is odd.

Returns
-------
    bool: True iff input number is odd
"
            ))])
        );
        Ok(())
    }
    #[test]
    fn parse_test_python_function_docstring() -> Result<()> {
        let program = parse_python_str(test_python_func_docstring())?;

        let documentation = extract_module_documentation(&program, false, false);
        // we checked before there is at least one class, so this is safe
        #[allow(clippy::unwrap_used)]
        let function = documentation.functions.first().unwrap();
        let docstring = function.docstring.clone();
        assert_eq!(
            docstring,
            Some(vec![ContentNode::Text(String::from(
                r"
Determine whether a number is odd.

Returns
-------
    bool: True iff input number is odd
"
            ))])
        );
        Ok(())
    }
}
