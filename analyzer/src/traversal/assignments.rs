use crate::context::{Context, Location};
use crate::errors::FatalError;
use crate::namespace::scopes::{BlockScope, Shared};
use crate::operations;
use crate::traversal::expressions;
use crate::traversal::utils::add_bin_operations_errors;
use fe_common::diagnostics::Label;
use fe_parser::ast as fe;
use fe_parser::node::Node;
use std::rc::Rc;

/// Gather context information for assignments and check for type errors.
///
/// e.g. `foo[42] = "bar"`, `self.foo[42] = "bar"`, `foo = 42`
pub fn assign(
    scope: Shared<BlockScope>,
    context: &mut Context,
    stmt: &Node<fe::FuncStmt>,
) -> Result<(), FatalError> {
    if let fe::FuncStmt::Assign { target, value } = &stmt.kind {
        let target_attributes = expressions::expr(Rc::clone(&scope), context, target, None)?;

        let value_attributes = expressions::expr(
            Rc::clone(&scope),
            context,
            value,
            Some(&target_attributes.typ),
        )?;
        check_assign_target(context, target)?;
        if target_attributes.typ != value_attributes.typ {
            context.fancy_error(
                "mismatched types",
                vec![
                    Label::primary(
                        target.span,
                        format!("this variable has type `{}`", target_attributes.typ),
                    ),
                    Label::secondary(
                        value.span,
                        format!(
                            "this value has incompatible type `{}`",
                            value_attributes.typ
                        ),
                    ),
                ],
                vec![],
            );
        }

        if matches!(
            (
                target_attributes.location,
                value_attributes.final_location(),
            ),
            (Location::Memory, Location::Storage { .. })
        ) {
            context.fancy_error(
                "location mismatch",
                vec![
                    Label::primary(target.span, "this variable is located in memory"),
                    Label::secondary(value.span, "this value is located in storage"),
                ],
                vec!["Hint: values located in storage can be copied to memory using the `to_mem` function.".into(),
                     "Example: `self.my_array.to_mem()`".into(),
                    ],
            );
        }

        return Ok(());
    }

    unreachable!()
}

pub fn check_assign_target(context: &mut Context, expr: &Node<fe::Expr>) -> Result<(), FatalError> {
    use fe::Expr::*;
    match &expr.kind {
        Attribute { .. } => Ok(()),
        Subscript { .. } => Ok(()),
        Tuple { elts } => {
            for elt in elts {
                check_assign_target(context, elt)?;
            }
            Ok(())
        }
        Name(_) => Ok(()),
        _ => {
            context.fancy_error("invalid assignment target",
                                             vec![Label::primary(expr.span, "")],
                                             vec!["The left side of an assignment can be a variable name, attribute, subscript, or tuple.".into()]);
            Err(FatalError)
        }
    }
}

/// Gather context information for assignments and check for type errors.
pub fn aug_assign(
    scope: Shared<BlockScope>,
    context: &mut Context,
    stmt: &Node<fe::FuncStmt>,
) -> Result<(), FatalError> {
    if let fe::FuncStmt::AugAssign { target, op, value } = &stmt.kind {
        check_assign_target(context, target)?;
        let target_attributes = expressions::expr(Rc::clone(&scope), context, target, None)?;
        let value_attributes =
            expressions::expr(scope, context, value, Some(&target_attributes.typ))?;

        if let Err(err) = operations::bin(&target_attributes.typ, &op.kind, &value_attributes.typ) {
            add_bin_operations_errors(context, target, value, err);
        }
        return Ok(());
    }

    unreachable!()
}
