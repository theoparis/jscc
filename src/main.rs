use boa_interner::Interner;
use boa_parser::{Parser, Source};
use jscc::CodeGenerator;
use llvm_sys::{
    analysis::LLVMVerifyModule,
    core::{LLVMBuildRetVoid, LLVMDisposeMessage, LLVMPrintModuleToString},
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMExecutionEngineRef, LLVMLinkInInterpreter,
        LLVMRunFunction,
    },
};

fn main() -> Result<(), String> {
    let js_code = "puts('Hello, World!\\n');";
    let js_code_bytes = js_code.as_bytes();
    let mut parser = Parser::new(Source::from_bytes(&js_code_bytes));
    let mut codegen = CodeGenerator::default();

    let mut interner = Interner::new();
    let ast = parser.parse_module(&mut interner).unwrap();

    let mut last_value = None;

    for module_item in ast.items().items() {
        last_value = codegen.compile_module_item(module_item, &interner);
    }

    let last_value = last_value.unwrap();

    unsafe {
        LLVMBuildRetVoid(codegen.context.builder);

        let ir = LLVMPrintModuleToString(codegen.context.module);
        println!("{}", std::ffi::CStr::from_ptr(ir).to_string_lossy());
        LLVMDisposeMessage(ir);

        let message = std::ptr::null_mut();
        LLVMVerifyModule(
            codegen.context.module,
            llvm_sys::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            message,
        );

        LLVMLinkInInterpreter();
        let mut engine: LLVMExecutionEngineRef = std::ptr::null_mut();

        let err = std::ptr::null_mut();

        LLVMCreateExecutionEngineForModule(&mut engine as *mut *mut _, codegen.context.module, err);

        let mut args = vec![];

        LLVMRunFunction(engine, codegen.context.root_function, 0, args.as_mut_ptr());
    }

    Ok(())
}
