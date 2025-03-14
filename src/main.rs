use std::fs::File;
use std::env;
use std::io::Read;

use llvm_tutorial::parser::parse;

fn main() -> std::io::Result<()>{
   
    let src_path = env::args().nth(1).ok_or(std::io::ErrorKind::NotFound)?;
    let mut src_file = File::open(src_path)?;
    let mut src = String::new();
    src_file.read_to_string(&mut src)?;

    if let Ok((remaining, ast)) = parse(src.as_str()){
        println!("AST:\n{:#?}\n", ast);
        println!("Unparsed:\n{:?}\n", remaining);
    }


    Ok(())
}

#[allow(dead_code)]
fn llvm_test()
    {
    unsafe {
        use llvm_sys::{ core::*, analysis::*, target::*};
        let context = LLVMContextCreate();

        let module_name = b"nop\0".as_ptr() as *const _;
        let module = LLVMModuleCreateWithNameInContext(module_name, context);
        let builder = LLVMCreateBuilderInContext(context);

        //Types
        let _void_type = LLVMVoidTypeInContext(context);
        let i32_type = LLVMIntTypeInContext(context, 32);
        let function_type = LLVMFunctionType(i32_type, std::ptr::null_mut(), 0, false.into());
        
        let function_name = b"entry\0".as_ptr();
        let function = LLVMAddFunction(module, function_name as *const _, function_type); 

        let block = LLVMAppendBasicBlockInContext(context, function, function_name as *const _);
        LLVMPositionBuilderAtEnd(builder, block);

        let val = LLVMConstInt(i32_type, 6, false.into());
        LLVMBuildRet(builder, val);

        LLVMDumpModule(module);

        let mut err_msg = std::mem::zeroed();
        let _res = LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMPrintMessageAction, &mut err_msg);
        LLVMDisposeMessage(err_msg);
        //assert!(res != 0, "Invalid Module!");

        LLVM_InitializeNativeTarget();
        //let object_file_name = CString::new("output.o").unwrap();
        //let object_file_name = object_file_name.as_bytes_with_nul();
        //assert!(llvm_sys::bit_writer::LLVMWriteBitcodeToFile(module, object_file_name[0] as *const _) != 0);

        LLVMDisposeBuilder(builder);
        LLVMDisposeModule(module);
        LLVMContextDispose(context);
    }
}
