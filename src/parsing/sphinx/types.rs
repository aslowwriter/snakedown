use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::eyre;
use lazy_regex::regex_captures;
use strum::EnumString;

#[derive(Debug, PartialEq)]
pub enum SphinxPriority {
    Omit,
    Standard,
    High,
    Low,
}

impl TryFrom<&str> for SphinxPriority {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "-1" => Ok(SphinxPriority::Omit),
            "1" => Ok(SphinxPriority::Standard),
            "0" => Ok(SphinxPriority::High),
            "2" => Ok(SphinxPriority::Low),
            _ => Err(eyre!("Unknown Priority: {}", value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SphinxType {
    Std(StdRole),
    C(CRole),
    Python(PyRole),
    Cpp(CppRole),
    JavaScript(JsRole),
    Mathematics(MathRole),
    ReStructuredText(RstRole),
}

impl TryFrom<&str> for SphinxType {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once(":") {
            Some((domain, role)) => match domain {
                "std" => Ok(SphinxType::Std(StdRole::from_str(role)?)),
                "c" => Ok(SphinxType::C(CRole::from_str(role)?)),
                "cpp" => Ok(SphinxType::Cpp(CppRole::from_str(role)?)),
                "py" => Ok(SphinxType::Python(PyRole::from_str(role)?)),
                "js" => Ok(SphinxType::JavaScript(JsRole::from_str(role)?)),
                "math" => Ok(SphinxType::Mathematics(MathRole::from_str(role)?)),
                "rst" => Ok(SphinxType::ReStructuredText(RstRole::from_str(role)?)),
                _ => Err(eyre!("Unknown domain: {}", domain)),
            },
            None => Err(eyre!("sphinx ref type did not contain :")),
        }
    }
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum StdRole {
    Doc,
    Label,
    Term,
    Cmdoption,
    Pdbcommand,
    Token,
    Opcode,
    MonitoringEvent,
    Envvar,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum CRole {
    Enum,
    Enumerator,
    Function,
    FunctionParam,
    Macro,
    Member,
    Type,
    Var,
    Struct,
    Union,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum CppRole {
    Class,
    Function,
    FunctionParam,
    Member,
    TemplateParam,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum JsRole {
    Module,
    Function,
    Method,
    Class,
    Data,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum MathRole {
    Numref,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum PyRole {
    Attribute,
    Data,
    Exception,
    Function,
    Method,
    Module,
    Property,
    Class,
}
#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum RstRole {
    Directive,
}

#[derive(Debug)]
pub struct ExternalSphinxRef {
    pub name: String,
    // type is a reserved keyword
    pub sphinx_type: SphinxType,
    pub priority: SphinxPriority,
    pub location: String,
    pub dispname: String,
}

impl TryFrom<&str> for ExternalSphinxRef {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some((_whole, name, sphinx_type, priority, location, dispname)) =
            regex_captures!(r"(.+?)\s+(\S+)\s+(-?\d+)\s+?(\S*)\s+(.*)", &value)
        {
            let display_name = if dispname == "-" {
                name.to_string()
            } else {
                dispname.to_string()
            };
            let location = location.replace("$", name);
            Ok(ExternalSphinxRef {
                name: name.to_owned(),
                sphinx_type: SphinxType::try_from(sphinx_type)?,
                priority: SphinxPriority::try_from(priority)?,
                location,
                dispname: display_name,
            })
        } else {
            Err(eyre!("failed to parse line"))
        }
    }
}

pub type Dispname = String;
pub type RelPath = PathBuf;

#[cfg(test)]
mod test {
    use color_eyre::Result;

    use super::*;

    #[test]
    fn test_sphinx_type_parsing_c() -> Result<()> {
        assert_eq!(SphinxType::try_from("c:enum")?, SphinxType::C(CRole::Enum));
        assert_eq!(
            SphinxType::try_from("c:enumerator")?,
            SphinxType::C(CRole::Enumerator)
        );
        assert_eq!(
            SphinxType::try_from("c:function")?,
            SphinxType::C(CRole::Function)
        );
        assert_eq!(
            SphinxType::try_from("c:functionParam")?,
            SphinxType::C(CRole::FunctionParam)
        );
        assert_eq!(
            SphinxType::try_from("c:macro")?,
            SphinxType::C(CRole::Macro)
        );
        assert_eq!(
            SphinxType::try_from("c:member")?,
            SphinxType::C(CRole::Member)
        );
        assert_eq!(SphinxType::try_from("c:type")?, SphinxType::C(CRole::Type));
        assert_eq!(SphinxType::try_from("c:var")?, SphinxType::C(CRole::Var));
        assert_eq!(
            SphinxType::try_from("c:struct")?,
            SphinxType::C(CRole::Struct)
        );
        assert_eq!(
            SphinxType::try_from("c:union")?,
            SphinxType::C(CRole::Union)
        );
        Ok(())
    }
    #[test]
    fn test_sphinx_type_parsing_cpp() -> Result<()> {
        assert_eq!(
            SphinxType::try_from("cpp:class")?,
            SphinxType::Cpp(CppRole::Class)
        );
        assert_eq!(
            SphinxType::try_from("cpp:function")?,
            SphinxType::Cpp(CppRole::Function)
        );
        assert_eq!(
            SphinxType::try_from("cpp:functionParam")?,
            SphinxType::Cpp(CppRole::FunctionParam)
        );
        assert_eq!(
            SphinxType::try_from("cpp:templateParam")?,
            SphinxType::Cpp(CppRole::TemplateParam)
        );
        assert_eq!(
            SphinxType::try_from("cpp:member")?,
            SphinxType::Cpp(CppRole::Member)
        );
        Ok(())
    }
    #[test]
    fn test_sphinx_type_parsing_math() -> Result<()> {
        assert_eq!(
            SphinxType::try_from("math:numref")?,
            SphinxType::Mathematics(MathRole::Numref)
        );
        Ok(())
    }
    #[test]
    fn test_sphinx_type_parsing_js() -> Result<()> {
        assert_eq!(
            SphinxType::try_from("js:module")?,
            SphinxType::JavaScript(JsRole::Module)
        );
        assert_eq!(
            SphinxType::try_from("js:function")?,
            SphinxType::JavaScript(JsRole::Function)
        );
        assert_eq!(
            SphinxType::try_from("js:method")?,
            SphinxType::JavaScript(JsRole::Method)
        );
        assert_eq!(
            SphinxType::try_from("js:class")?,
            SphinxType::JavaScript(JsRole::Class)
        );
        assert_eq!(
            SphinxType::try_from("js:data")?,
            SphinxType::JavaScript(JsRole::Data)
        );
        Ok(())
    }
    #[test]
    fn test_sphinx_type_parsing_py() -> Result<()> {
        assert_eq!(
            SphinxType::try_from("py:attribute")?,
            SphinxType::Python(PyRole::Attribute)
        );
        assert_eq!(
            SphinxType::try_from("py:data")?,
            SphinxType::Python(PyRole::Data)
        );
        assert_eq!(
            SphinxType::try_from("py:exception")?,
            SphinxType::Python(PyRole::Exception)
        );
        assert_eq!(
            SphinxType::try_from("py:function")?,
            SphinxType::Python(PyRole::Function)
        );
        assert_eq!(
            SphinxType::try_from("py:method")?,
            SphinxType::Python(PyRole::Method)
        );
        assert_eq!(
            SphinxType::try_from("py:module")?,
            SphinxType::Python(PyRole::Module)
        );
        assert_eq!(
            SphinxType::try_from("py:property")?,
            SphinxType::Python(PyRole::Property)
        );
        assert_eq!(
            SphinxType::try_from("py:class")?,
            SphinxType::Python(PyRole::Class)
        );
        Ok(())
    }
    #[test]
    fn test_sphinx_type_parsing_std() -> Result<()> {
        assert_eq!(
            SphinxType::try_from("std:doc")?,
            SphinxType::Std(StdRole::Doc)
        );
        assert_eq!(
            SphinxType::try_from("std:label")?,
            SphinxType::Std(StdRole::Label)
        );
        assert_eq!(
            SphinxType::try_from("std:term")?,
            SphinxType::Std(StdRole::Term)
        );
        Ok(())
    }
}
