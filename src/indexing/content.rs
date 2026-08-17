use winnow::{
    ModalResult, Parser,
    combinator::{alt, delimited, eof, repeat_till, trace},
    stream::AsChar,
    token::{rest, take_till, take_until},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Reference {
    pub fully_qualified_name: String,
    pub display_text: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ValidatedReference {
    pub subject: String,
    pub display_text: String,
}

impl Reference {
    pub fn new(name: String, display: Option<String>) -> Self {
        Self {
            fully_qualified_name: name,
            display_text: display,
        }
    }

    pub fn original(&self) -> String {
        match &self.display_text {
            None => {
                format!("[[{}]]", self.fully_qualified_name)
            }
            Some(d) => format!("[[{}|{}]]", self.fully_qualified_name, d),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ContentNode {
    Text(String),
    Reference(Reference),
}

fn reference(input: &mut &str) -> ModalResult<ContentNode> {
    let text = trace(
        "reference",
        delimited(
            "[[",
            alt((
                take_until(0.., "]]").verify(|s: &str| !s.contains('\n')),
                take_till(1.., AsChar::is_newline),
            )),
            "]]",
        ),
    )
    .parse_next(input)?;

    if let Some((name, disp_text)) = text.split_once('|') {
        Ok(ContentNode::Reference(Reference::new(
            name.trim().to_string(),
            Some(disp_text.trim().to_string()),
        )))
    } else {
        Ok(ContentNode::Reference(Reference::new(
            text.trim().to_string(),
            None,
        )))
    }
}

fn text_node(input: &mut &str) -> ModalResult<ContentNode> {
    let text = trace("text_node", alt((take_until(1.., "[["), rest))).parse_next(input)?;
    Ok(ContentNode::Text(text.to_string()))
}

pub(crate) fn parse_contents(input: &mut &str) -> ModalResult<Vec<ContentNode>> {
    let (content, _) = repeat_till(0.., alt((reference, text_node)), eof).parse_next(input)?;

    Ok(content)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_empty_string() {
        let mut input = "";
        let content = parse_contents(&mut input);

        assert_eq!(content, Ok(vec![]));
    }
    #[test]
    fn test_only_ref() -> ModalResult<()> {
        let mut input = "[[foo.bar]]";
        let content = reference(&mut input)?;

        assert_eq!(
            content,
            ContentNode::Reference(Reference::new("foo.bar".to_string(), None))
        );
        Ok(())
    }
    #[test]
    fn test_starts_with_text_has_ref() {
        let mut input = "lorem ipsum. [[foo.bar]]";
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![
                ContentNode::Text(String::from("lorem ipsum. ")),
                ContentNode::Reference(Reference::new(String::from("foo.bar"), None)),
            ])
        );
    }
    #[test]
    fn test_starts_with_reference_has_text() {
        let mut input = "[[foo.bar]] lorem ipsum.";
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![
                ContentNode::Reference(Reference::new(String::from("foo.bar"), None)),
                ContentNode::Text(String::from(" lorem ipsum."))
            ])
        );
    }
    #[test]
    fn test_text() {
        let mut input = "lorem ipsum.";
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![ContentNode::Text(String::from("lorem ipsum."))])
        );
    }

    #[test]
    fn test_nasty_non_reference() {
        let mut input = r#"[[foo == bar] = baz]"#;
        let content = text_node(&mut input);

        assert_eq!(
            content,
            Ok(ContentNode::Text(String::from(r#"[[foo == bar] = baz]"#)))
        );
    }

    #[test]
    fn test_full_docstring() {
        let mut input = r#"
Generate a greeting message. This is a method on the [[test_pkg.bar.Greeter]] class.
It is distinct from the [[test_pkg.bar.greet]] function.

Returns:
    str: Greeting message.

See Also:
    [[ foo.bar ]]
    [[ bar.baz ]]
    [[ arf.mew ]]
"#;
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![
                ContentNode::Text(String::from(
                    r#"
Generate a greeting message. This is a method on the "#
                )),
                ContentNode::Reference(Reference::new(String::from("test_pkg.bar.Greeter"), None)),
                ContentNode::Text(String::from(
                    r#" class.
It is distinct from the "#
                )),
                ContentNode::Reference(Reference::new(String::from("test_pkg.bar.greet"), None)),
                ContentNode::Text(String::from(
                    r#" function.

Returns:
    str: Greeting message.

See Also:
    "#
                )),
                ContentNode::Reference(Reference::new(String::from("foo.bar"), None)),
                ContentNode::Text(String::from(
                    r#"
    "#
                )),
                ContentNode::Reference(Reference::new(String::from("bar.baz"), None)),
                ContentNode::Text(String::from(
                    r#"
    "#
                )),
                ContentNode::Reference(Reference::new(String::from("arf.mew"), None)),
                ContentNode::Text(String::from(
                    r#"
"#
                )),
            ])
        );
    }

    #[test]
    fn test_ref_with_interneal_newline() {
        let mut input = r#"[[foo.
bar]]lorem ipsum"#;
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![ContentNode::Text(String::from(
                "[[foo.\nbar]]lorem ipsum"
            ))])
        );
    }
    #[test]
    fn test_nasty() {
        let mut input = r#"[[], None]
just a r▐
andom closure to make the types interesting to render."#;
        let content = parse_contents(&mut input);

        assert_eq!(
            content,
            Ok(vec![ContentNode::Text(String::from(
                "[[], None]\njust a r▐\nandom closure to make the types interesting to render."
            ))])
        );
    }

    use pretty_assertions::assert_eq;
    //

    #[test]
    fn reference_parsing() -> ModalResult<()> {
        let mut test_text = r#"
    mod {test}
    [[]]
[[greeter]]
[[foo.bar]]
[[_foo.bar]]
[[_foo]]
[[_]]
[[asdf.asdf.asdf|display text]]
[[       foo.bar     ]]
[[      asdf.asdf.asdf       |     |||display text|||      ]]
idx[[foo == bar] = baz]
asdlkfj;alskdj;alsdkj
askdfjoiw3fmxj,cavuiw43i
        "#;

        let expected_refs = vec![
            ("", None),
            ("greeter", None),
            ("foo.bar", None),
            ("_foo.bar", None),
            ("_foo", None),
            ("_", None),
            ("asdf.asdf.asdf", Some("display text".to_string())),
            ("foo.bar", None),
            ("asdf.asdf.asdf", Some("|||display text|||".to_string())),
        ]
        // just so I don't have to type out ObjectRef::new every time
        .into_iter()
        .map(|tup| ContentNode::Reference(Reference::new(tup.0.to_owned(), tup.1)))
        .collect::<Vec<ContentNode>>();

        let found_refs: Vec<_> = parse_contents(&mut test_text)?
            .into_iter()
            .filter(|n| matches!(n, ContentNode::Reference(_)))
            .collect();

        assert_eq!(expected_refs, found_refs);

        Ok(())
    }
}
