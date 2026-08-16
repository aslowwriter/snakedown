use rustpython_parser::ast::{CmpOp, Comprehension, Constant, Expr, Keyword, Operator, UnaryOp};

use super::args::render_args;

pub fn render_expr(expr: Expr) -> String {
    let mut out = String::new();

    #[allow(unused_variables)]
    match expr {
        Expr::BoolOp(expr_bool_op) => {
            let op = match expr_bool_op.op {
                rustpython_parser::ast::BoolOp::And => " and ",
                rustpython_parser::ast::BoolOp::Or => " or ",
            };
            out.push_str(
                &expr_bool_op
                    .values
                    .into_iter()
                    .map(render_expr)
                    .collect::<Vec<_>>()
                    .join(op),
            );
        }
        Expr::NamedExpr(expr_named_expr) => {
            out.push('(');
            out.push_str(&render_expr(*expr_named_expr.target));
            out.push_str(" := ");
            out.push_str(&render_expr(*expr_named_expr.value));
            out.push(')');
        }
        Expr::BinOp(expr_bin_op) => {
            out.push_str(&render_expr(*expr_bin_op.left));
            out.push(' ');
            out.push_str(render_operator(expr_bin_op.op));
            out.push(' ');
            out.push_str(&render_expr(*expr_bin_op.right));
        }
        Expr::UnaryOp(expr_unary_op) => {
            out.push_str(render_unaryop(expr_unary_op.op));
            out.push_str(&render_expr(*expr_unary_op.operand));
        }
        Expr::Lambda(expr_lambda) => {
            out.push_str("lambda ");
            out.push_str(&render_args(*expr_lambda.args));
            out.push_str(": ");
            out.push_str(&render_expr(*expr_lambda.body));
        }
        Expr::IfExp(expr_if_exp) => {
            out.push_str(&render_expr(*expr_if_exp.body));
            out.push_str(" if ");
            out.push_str(&render_expr(*expr_if_exp.test));
            out.push_str(" else ");
            out.push_str(&render_expr(*expr_if_exp.orelse));
        }
        Expr::Dict(expr_dict) => {
            out.push('{');
            out.push_str(
                &expr_dict
                    .keys
                    .iter()
                    .zip(expr_dict.values)
                    .map({
                        |(k, v)| {
                            if let Some(key) = k {
                                format!("{}: {}", render_expr(key.clone()), render_expr(v))
                            } else {
                                format!("**{}", render_expr(v))
                            }
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push('}');
        }
        Expr::Set(expr_set) => {
            out.push('{');
            out.push_str(
                &expr_set
                    .elts
                    .into_iter()
                    .map(render_expr)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push('}');
        }
        Expr::ListComp(expr_list_comp) => {
            out.push('[');
            out.push_str(&render_expr(*expr_list_comp.elt));
            out.push(' ');
            out.push_str(
                &expr_list_comp
                    .generators
                    .into_iter()
                    .map(render_comprehension)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            out.push(']');
        }
        Expr::DictComp(expr_dict_comp) => {
            out.push('{');
            out.push_str(&render_expr(*expr_dict_comp.key));
            out.push_str(": ");
            out.push_str(&render_expr(*expr_dict_comp.value));
            out.push(' ');
            out.push_str(
                &expr_dict_comp
                    .generators
                    .into_iter()
                    .map(render_comprehension)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            out.push('}');
        }
        Expr::SetComp(expr_set_comp) => {
            out.push('{');
            out.push_str(&render_expr(*expr_set_comp.elt));
            out.push(' ');
            out.push_str(
                &expr_set_comp
                    .generators
                    .into_iter()
                    .map(render_comprehension)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            out.push('}');
        }
        Expr::GeneratorExp(expr_generator_exp) => {
            out.push('(');
            out.push_str(&render_expr(*expr_generator_exp.elt));
            out.push(' ');
            out.push_str(
                &expr_generator_exp
                    .generators
                    .into_iter()
                    .map(render_comprehension)
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            out.push(')');
        }
        Expr::Await(expr_await) => {
            out.push_str("await ");
            out.push_str(&render_expr(*expr_await.value));
        }
        Expr::Yield(expr_yield) => {
            out.push_str("yield ");
            if let Some(val) = expr_yield.value {
                out.push_str(&render_expr(*val));
            }
        }
        Expr::YieldFrom(expr_yield_from) => {
            out.push_str("yield from ");
            out.push_str(&render_expr(*expr_yield_from.value));
        }
        Expr::Compare(expr_compare) => {
            out.push_str(&render_expr(*expr_compare.left));
            expr_compare
                .ops
                .iter()
                .zip(expr_compare.comparators)
                .for_each(|(op, expr)| {
                    out.push(' ');
                    out.push_str(render_cmp_op(op));
                    out.push(' ');
                    out.push_str(&render_expr(expr))
                });
        }
        Expr::Call(expr_call) => {
            out.push_str(&render_expr(*expr_call.func));
            out.push('(');
            out.push_str(
                &expr_call
                    .args
                    .into_iter()
                    .map(render_expr)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            if !expr_call.keywords.is_empty() {
                out.push_str(", ");
                out.push_str(
                    &expr_call
                        .keywords
                        .into_iter()
                        .map(render_keyword)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
            }
            out.push(')');
        }
        Expr::FormattedValue(expr_formatted_value) => todo!(),
        Expr::JoinedStr(expr_joined_str) => todo!(),
        Expr::Constant(expr_constant) => out.push_str(&render_constant(expr_constant.value)),
        Expr::Attribute(expr_attribute) => {
            out.push_str(&format!(
                "{}.{}",
                render_expr(*expr_attribute.value),
                expr_attribute.attr
            ));
        }
        Expr::Subscript(expr_subscript) => {
            let rendered_slice = render_expr(*expr_subscript.slice);
            let rendered_slice_clean = rendered_slice
                .strip_prefix("(")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(&rendered_slice);
            out.push_str(&format!(
                "{}[{}]",
                render_expr(*expr_subscript.value),
                rendered_slice_clean
            ));
        }
        Expr::Starred(expr_starred) => {
            out.push('*');
            out.push_str(&render_expr(*expr_starred.value));
        }
        Expr::Name(expr_name) => out.push_str(expr_name.id.as_str()),
        Expr::List(expr_list) => {
            out.push('[');
            out.push_str(
                &expr_list
                    .elts
                    .into_iter()
                    .map(render_expr)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push(']');
        }
        Expr::Tuple(expr_tuple) => {
            out.push('(');
            out.push_str(
                &expr_tuple
                    .elts
                    .into_iter()
                    .map(render_expr)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push(')');
        }
        Expr::Slice(expr_slice) => {
            if let Some(lower) = expr_slice.lower {
                out.push_str(&render_expr(*lower));
            }
            out.push(':');
            if let Some(upper) = expr_slice.upper {
                out.push_str(&render_expr(*upper));
            }
        }
    }

    out.trim().to_string()
}

fn render_comprehension(comp: Comprehension) -> String {
    let mut out = String::new();
    out.push_str("for ");
    out.push_str(&render_expr(comp.target));
    out.push_str(" in ");
    out.push_str(&render_expr(comp.iter));

    out
}

fn render_keyword(keyword: Keyword) -> String {
    let mut out = String::new();
    let fmt = if let Some(arg) = keyword.arg {
        format!("{}={}", arg, render_expr(keyword.value))
    } else {
        format!("**{}", render_expr(keyword.value))
    };
    out.push_str(&fmt);

    out
}

pub(crate) fn render_constant(constant: Constant) -> String {
    match constant {
        Constant::None => String::from("None"),
        Constant::Bool(b) => {
            if b {
                String::from("True")
            } else {
                String::from("False")
            }
        }
        Constant::Str(s) => format!("\"{s}\""),
        Constant::Bytes(_) => todo!(),
        Constant::Int(big_int) => format!("{big_int}"),
        Constant::Tuple(constants) => format!(
            "({})",
            constants
                .into_iter()
                .map(render_constant)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Constant::Float(f) => format!("{f}"),
        Constant::Complex { real, imag } => {
            if real.abs() < f64::EPSILON {
                format!("{imag}j")
            } else if imag.abs() < f64::EPSILON {
                format!("{real}")
            } else {
                format!("{real}+{imag}j")
            }
        }
        Constant::Ellipsis => String::from("..."),
    }
}

fn render_operator(op: Operator) -> &'static str {
    match op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mult => "*",
        Operator::MatMult => "@",
        Operator::Div => "/",
        Operator::Mod => "%",
        Operator::Pow => "**",
        Operator::LShift => "<<",
        Operator::RShift => ">>",
        Operator::BitOr => "|",
        Operator::BitXor => "^",
        Operator::BitAnd => "&",
        Operator::FloorDiv => "//",
    }
}

fn render_cmp_op(op: &CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "=",
        CmpOp::NotEq => "!=",
        CmpOp::Lt => "<",
        CmpOp::LtE => "<=",
        CmpOp::Gt => ">",
        CmpOp::GtE => ">=",
        CmpOp::Is => "is ",
        CmpOp::IsNot => "is not ",
        CmpOp::In => "in ",
        CmpOp::NotIn => "not in ",
    }
}

fn render_unaryop(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Invert => "~",
        UnaryOp::Not => "not ",
        UnaryOp::UAdd => "+",
        UnaryOp::USub => "-",
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use rustpython_parser::{
        Mode,
        ast::{ExprContext, ExprName, ExprYield, ExprYieldFrom, Identifier, Mod},
        parse,
        text_size::TextRange,
    };

    fn get_expr(s: &str) -> Result<Expr> {
        let parsed = parse(s, Mode::Expression, "<embedded>")?;
        if let Mod::Expression(mod_expr) = parsed {
            let expr = *mod_expr.body.clone();

            Ok(expr)
        } else {
            panic!()
        }
    }

    #[test]
    fn test_render_name() -> Result<()> {
        let s = "a";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }

    #[test]
    fn test_render_int() -> Result<()> {
        let s = "24";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_none() -> Result<()> {
        let s = "None";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bool_true() -> Result<()> {
        let s = "True";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bool_false() -> Result<()> {
        let s = "False";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_float() -> Result<()> {
        let s = "24.242424";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_ellipses() -> Result<()> {
        let s = "...";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_str() -> Result<()> {
        let s = "\"24\"";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_tuple() -> Result<()> {
        let s = "(24, 3)";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_binary_op_min() -> Result<()> {
        let s = "24 - 3";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_binary_op_div() -> Result<()> {
        let s = "24 / 3";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_empty_tuple() -> Result<()> {
        let s = "()";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_complex_tuple() -> Result<()> {
        let s = "(3 + 5j, True)";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_list() -> Result<()> {
        let s = "[24, 3]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_empty_list() -> Result<()> {
        let s = "[]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_complex_list() -> Result<()> {
        let s = "[3 + 5j, True]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_set() -> Result<()> {
        let s = "{24, 3}";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_empty_dict() -> Result<()> {
        let s = "{}";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_complex_set() -> Result<()> {
        let s = "{3 + 5j, True}";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_complex() -> Result<()> {
        let s = "3 + 5j";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_lshift() -> Result<()> {
        let s = "3 << 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_rshift() -> Result<()> {
        let s = "3 >> 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_mult() -> Result<()> {
        let s = "3 * 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_mod() -> Result<()> {
        let s = "3 % 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_pow() -> Result<()> {
        let s = "3 ** 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bit_or() -> Result<()> {
        let s = "3 | 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bit_and() -> Result<()> {
        let s = "3 & 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bit_xor() -> Result<()> {
        let s = "3 ^ 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bit_floor_div() -> Result<()> {
        let s = "3 // 5";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_complex_img_only() -> Result<()> {
        let s = "5j";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_list_comp() -> Result<()> {
        let s = "[a for a in b]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_dict() -> Result<()> {
        let s = "{a: 1, **d}";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_call() -> Result<()> {
        let s = "foo(bar, *baz, mew=bark, chirp=squeek, **kwargs)";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_subscript() -> Result<()> {
        let s = "foo[bar]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_subscript_slice() -> Result<()> {
        let s = "foo[1:3]";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_attribute() -> Result<()> {
        let s = "foo.bar";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_bool_and() -> Result<()> {
        let s = "True and False or True";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_render_matmul() -> Result<()> {
        let s = "a @ b";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_complex_real_only() -> Result<()> {
        let constant = Constant::Complex {
            real: 2.0,
            imag: 0.0,
        };
        let rendered = render_constant(constant);
        assert_eq!(rendered, "2");
        Ok(())
    }
    #[test]
    fn test_complex_real_and_img() -> Result<()> {
        let constant = Constant::Complex {
            real: 2.0,
            imag: 2.0,
        };
        let rendered = render_constant(constant);
        assert_eq!(rendered, "2+2j");
        Ok(())
    }
    #[test]
    fn test_constant_tuple() -> Result<()> {
        let constant = Constant::Tuple(vec![Constant::Bool(true), Constant::Bool(false)]);
        let rendered = render_constant(constant);
        assert_eq!(rendered, "(True, False)");
        Ok(())
    }
    #[test]
    fn test_unary_op_not() -> Result<()> {
        let s = "not True";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_unary_op_inv() -> Result<()> {
        {
            let s = "~arr";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_unary_op_usub() -> Result<()> {
        {
            let s = "-arr";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_unary_op_uadd() -> Result<()> {
        {
            let s = "+arr";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_named_expr() -> Result<()> {
        {
            let s = "(x := 4)";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_lambda() -> Result<()> {
        {
            let s = "lambda x: x ** 2";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_if_expr() -> Result<()> {
        {
            let s = "a if b else c";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_dict_compt() -> Result<()> {
        {
            let s = "{i: None for i in range(12)}";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_set_compr() -> Result<()> {
        {
            let s = "{a for a in [1, 2, 3]}";
            let expr = get_expr(s)?;

            let rendered = render_expr(expr);

            assert_eq!(rendered, s);

            Ok(())
        }
    }
    #[test]
    fn test_generator() -> Result<()> {
        let s = "{a for a in [1, 2, 3]}";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_await() -> Result<()> {
        let s = "await foo";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_yield() -> Result<()> {
        let yield_expr = Expr::Yield(ExprYield {
            range: TextRange::new(0.into(), 0.into()),
            value: Some(Box::new(Expr::Name(ExprName {
                range: TextRange::new(0.into(), 0.into()),
                id: Identifier::new("foo"),
                ctx: ExprContext::Load,
            }))),
        });

        let rendered = render_expr(yield_expr);

        assert_eq!(rendered, "yield foo");

        Ok(())
    }
    #[test]
    fn test_yield_from() -> Result<()> {
        let yield_expr = Expr::YieldFrom(ExprYieldFrom {
            range: TextRange::new(0.into(), 0.into()),
            value: Box::new(Expr::Name(ExprName {
                range: TextRange::new(0.into(), 0.into()),
                id: Identifier::new("foo"),
                ctx: ExprContext::Load,
            })),
        });

        let rendered = render_expr(yield_expr);

        assert_eq!(rendered, "yield from foo");

        Ok(())
    }
    #[test]
    fn test_compare() -> Result<()> {
        let s = "1 < a <= 10";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_simple_call() -> Result<()> {
        let s = "range(12)";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
    #[test]
    fn test_generator_exp() -> Result<()> {
        let s = "(a for a in range(12))";
        let expr = get_expr(s)?;

        let rendered = render_expr(expr);

        assert_eq!(rendered, s);

        Ok(())
    }
}
