use std::collections::HashMap;
use crate::{ast, types};

const FRAME_BUFFER_ID: &str = "frame_buffer";
const CONTINUE_FLAG_ID: &str = "flag";

const ERROR_LABEL: &str = "suspend";

pub fn emit<I>(ast: ast::Story<I>) -> String {
    let ast::Story(root, knots, functions) = ast;
    let string_table = HashMap::<(String, String, usize), String>::new();
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

        let mut start_index = 0;
        let mut contianer_index = 0;
        let mut body_asm = String::new();
        for weave in stitch.body {
            let loop_ast = weave.content.map_while(|content|
                //TODO: Add delimiters as nessicary
                match content { 
                    ast::Content::Text(text) => Some(text),
                    ast::Content::Newline => Some("\n"),
                    _ => None
                }
            );

            let loop_strings = loop_ast.enumerate().map(
                |i, content| {
                    /* TODO: Do I need a string table?
                    string_table.insert( 
                        (stitch.signature.name, weave.label, i), 
                        content
                    );
                    */
                    (
                        ["content", &stitch.signature.name, weave.label, i].join("."),
                        content.len()+1,
                        content
                    )

                }
            );

            let loop_string_constants = loop_strings.map(
                |label, length, content| format!(
                    "@{} = constant [{} x i8] c\"{}\\00\"", 
                    label, length, content
                )
            );

            let loop_string_array_id = ["strings", &stitch.signature.name, weave.label, contianer_index].join(".");
            let loop_strings_array_asm = format!("@{} = constant [{} x ptr] [{}]",
                loop_string_array_id,
                loop_strings.len(),
                loop_strings.map(
                    |label, _length, _content| format!("ptr @{}", label) 
                ).join(", ").collect::<String>()
            );
            contianer_index += 1;

            if let Some(loop_ast) = loop_ast {
                start_index += loop_strings.len();
                let loop_asm = emit_loop(contianer_index, start_index, start_index+loop_strings.len(), todo!(), todo!(), loop_string_array_id);
                body_asm.push_str(&loop_asm);
            };

        };

        let function_head = "----\nTODO\n----";
        format!("define {} presplitcoroutine noinline\n{{\n{}\n{}\n}}", signature_asm, function_head, body_asm)
    }).collect::<String>();

    body_asm
}

pub fn emit_type(ty: types::Value) -> String {
    match ty {
        types::Value::Integer(_) => String::from("i32"),
        types::Value::Decimal(_) => String::from("f32"),
        types::Value::String(_) => String::from("%string_type"),
        types::Value::Bool(_) => String::from("i1"),
        types::Value::Divert => todo!(), //String::from(""), 
        types::Value::ListValue => todo!(), //String::from(""),
    }
}

fn emit_loop(i: usize, start_index: usize, end_index: usize, entry_label: String, exit_label: String, string_array_id: String) -> String {
    let loop_label = format!("loop_{}", i);
    let resume_label = format!("resume.loop_{}", i);
    let suspend_label = format!("suspend_point.loop_{}", i);

    let loop_block = format!("{4}:
        %index_{0} = phi i32 [{1}, {2}], [%inc_{0}, %resume.loop_{0}], [%inc_{0}, %suspend_point.loop_{0}]
        %string_{0}.addr = getelementptr [2 x ptr], ptr @{3}, i32 0, i32 %index_{0}
        %string_{0} = load ptr, ptr %string_{0}.addr
        call i32 @write_string(ptr @out_stream, ptr %string_{0})", 

        i, start_index, entry_label, string_array_id, loop_label
    );
    let incriment_asm = format!("
        %inc_{0} = add i32 %index_{0}, {2}
        %cond_{0} =	icmp ule i32 %inc_{0}, {2}
        br i1 %cond_{0}, label %resume.loop_{0}, label %{1}",
        
        i, exit_label, end_index
    );
    
    let resume_block = format!("{1}:
        %continue_value_{0} = load i1, ptr @continue_maximally
        br i1 %continue_value_{0}, label %{2}, label %{3}",

        i, resume_label, loop_label, suspend_label
    );

    let suspend_block = format!("{2}
        %resume_abnormal_loop_{0} = call i1 (...) @llvm.coro.suspend.retcon.i1(%yield_type {{i32 0, i1 true}})
        br i1 %resume_abnormal_loop_{0}, label %{3}, label %{1}",
        i, loop_label, suspend_label, ERROR_LABEL
    );

    format!("{}\n{}\n{}\n{}", loop_block, incriment_asm, resume_block, suspend_block)
}
