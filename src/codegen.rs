use crate::{ast, types};

const FRAME_BUFFER_ID: &str = "frame_buffer";
const CONTINUE_FLAG_ID: &str = "flag";
pub fn emit<I>(ast: ast::Story<I>) -> String {
    let ast::Story(root, knots, functions) = ast;

    let body_asm = root.body.iter().map(|stitch| {
        let signature_asm = format!(
            "{} @{}(ptr %{}, i1 %{}{})", 
            if let Some(ty) = stitch.signature.ret.clone() {
                emit_type(ty)
            } else {
                "void".to_string()
            },
            stitch.signature.name, 
            FRAME_BUFFER_ID, CONTINUE_FLAG_ID, 
            stitch.signature.parameters.iter().map(|param|format!(", ptr %{}", param.name)).collect::<String>()
        );

        format!("declare {0}\ndefine {0} presplitcoroutine noinline", signature_asm)
    }).collect::<String>();

    body_asm
}

pub fn emit_type(ty: types::Value) -> String {
    match ty {
        types::Value::Integer(_) => String::from("i32"),
        types::Value::Decimal(_)   => String::from("f32"),
        types::Value::String(_) => String::from("%string_type"),
        types::Value::Bool(_)     => String::from("i1"),
        types::Value::Divert         => todo!(), //String::from(""), 
        types::Value::ListValue      => todo!(), //String::from(""),
    }
}
