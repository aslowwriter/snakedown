use crate::indexing::content::ContentNode;
use crate::parsing::python::class::ValidatedClassDocumentation;
use crate::parsing::python::function::ValidatedFunctionDocumentation;
use crate::parsing::python::module::ValidatedModuleDocumentation;

use super::class::ClassDocumentation;
use super::function::FunctionDocumentation;
use super::module::ModuleDocumentation;

#[derive(Debug)]
pub enum ValidatedObjectDocumentation {
    Module(ValidatedModuleDocumentation),
    Class(ValidatedClassDocumentation),
    Function(ValidatedFunctionDocumentation),
}
#[derive(Debug)]
pub enum ObjectDocumentation {
    Module(ModuleDocumentation),
    Class(ClassDocumentation),
    Function(FunctionDocumentation),
}

impl ObjectDocumentation {
    pub fn docstring_mut(&mut self) -> Option<&mut Vec<ContentNode>> {
        match self {
            ObjectDocumentation::Module(module_documentation) => {
                module_documentation.docstring.as_mut()
            }
            ObjectDocumentation::Class(class_documentation) => {
                class_documentation.docstring.as_mut()
            }
            ObjectDocumentation::Function(function_documentation) => {
                function_documentation.docstring.as_mut()
            }
        }
    }
    pub fn docstring(&self) -> Option<Vec<ContentNode>> {
        match self {
            ObjectDocumentation::Module(module_documentation) => {
                module_documentation.docstring.clone()
            }
            ObjectDocumentation::Class(class_documentation) => {
                class_documentation.docstring.clone()
            }
            ObjectDocumentation::Function(function_documentation) => {
                function_documentation.docstring.clone()
            }
        }
    }
}
