use crate::lowering::utils::ZeroSpanNode;
use fe_analyzer::namespace::types::{Array, Base, FixedSize, Integer, SafeNames, Tuple};
use fe_parser::{ast as fe, node::Node};
use vec1::Vec1;

/// The name of a lowered list expression generator function.
pub fn list_expr_generator_fn_name(list_expr_type: &Array) -> String {
    format!("list_expr_{}", list_expr_type.lower_snake())
}

/// The name of a lowered tuple struct definition.
pub fn tuple_struct_string(tuple: &Tuple) -> String {
    tuple.lower_snake()
}

/// The type description of a lowered tuple struct.
pub fn tuple_struct_type_desc(tuple: &Tuple) -> fe::TypeDesc {
    fe::TypeDesc::Base {
        base: tuple_struct_string(tuple),
    }
}

/// The name of a lowered tuple struct definition as an expression.
pub fn tuple_struct_name(tuple: &Tuple) -> fe::Expr {
    fe::Expr::Name(tuple_struct_string(tuple))
}

/// Maps a FixedSize type to its type description.
pub fn fixed_size_type_desc(typ: &FixedSize) -> fe::TypeDesc {
    match typ {
        FixedSize::Base(Base::Unit) => fe::TypeDesc::Unit,
        FixedSize::Base(base) => fe::TypeDesc::Base {
            base: base_type_name(base),
        },
        FixedSize::Array(array) => fe::TypeDesc::Array {
            dimension: array.size,
            typ: fixed_size_type_desc(&array.inner.clone().into()).into_boxed_node(),
        },
        FixedSize::Tuple(tuple) => fe::TypeDesc::Tuple {
            items: {
                let mut v1: Vec1<Node<fe::TypeDesc>> =
                    Vec1::new(fixed_size_type_desc(tuple.items.first()).into_node());
                for item in tuple.items.iter().skip(1) {
                    v1.push(fixed_size_type_desc(&item).into_node())
                }
                Vec1::from(v1)
            },
        },
        FixedSize::String(_) => todo!(),
        FixedSize::Contract(_) => todo!(),
        FixedSize::Struct(_) => todo!(),
    }
}

fn base_type_name(typ: &Base) -> String {
    match typ {
        Base::Numeric(number) => match number {
            Integer::U256 => "u256",
            Integer::U128 => "u128",
            Integer::U64 => "u64",
            Integer::U32 => "u32",
            Integer::U16 => "u16",
            Integer::U8 => "u8",
            Integer::I256 => "i256",
            Integer::I128 => "i128",
            Integer::I64 => "i64",
            Integer::I32 => "i32",
            Integer::I16 => "i16",
            Integer::I8 => "i8",
        },
        Base::Bool => "bool",
        Base::Byte => unimplemented!("byte should be removed"),
        Base::Address => "address",
        Base::Unit => "unit",
    }
    .to_string()
}
