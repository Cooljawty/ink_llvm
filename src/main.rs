fn main() {
    unsafe {
        use llvm_sys::{ core::*, analysis::*};
        let context = LLVMContextCreate();

        let module_name = b"nop\0".as_ptr() as *const _;
        let module = LLVMModuleCreateWithNameInContext(module_name, context);
        let builder = LLVMCreateBuilderInContext(context);

        //Types
        let void_type = LLVMVoidTypeInContext(context);
        let i32_type = LLVMIntTypeInContext(context, 32);
        let function_type = LLVMFunctionType(i32_type, std::ptr::null_mut(), 0, false.into());
        
        let function_name = b"entry\0".as_ptr();
        let function = LLVMAddFunction(module, module_name, function_type); 

        let block = LLVMAppendBasicBlockInContext(context, function, function_name as *const _);
        LLVMPositionBuilderAtEnd(builder, block);

        let val = LLVMConstInt(i32_type, 6, false.into());
        LLVMBuildRet(builder, val);

        LLVMVerifyFunction(function, LLVMVerifierFailureAction::LLVMAbortProcessAction);

        LLVMDumpModule(module);

        LLVMDisposeBuilder(builder);
        LLVMDisposeModule(module);
        LLVMContextDispose(context);
    }
}
