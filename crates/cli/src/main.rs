use boa_interner::Interner;
use boa_parser::{Parser, Source};
use clap::Parser as _;
use jscc_codegen::CodeGenerator;
use llvm_sys::{analysis::*, core::*, target::*, target_machine::*};
use std::{ffi::CString, path::PathBuf};

#[derive(clap::Parser)]
struct Cli {
    #[clap(short, long)]
    input_file: PathBuf,

    #[clap(short, long)]
    output_file: Option<PathBuf>,

    #[clap(short, long)]
    target: Option<String>,

    #[clap(short, long)]
    linker: Option<String>,

    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<(), String> {
    let args = Cli::parse();

    let js_file = std::fs::File::open(
        args.input_file
            .to_str()
            .expect("Could not convert path to string"),
    )
    .expect("Could not open file");

    let mut parser = Parser::new(Source::from_reader(&js_file, Some(&args.input_file)));
    let mut codegen = CodeGenerator::default();

    let mut interner = Interner::new();
    let ast = parser.parse_module(&mut interner).unwrap();

    for module_item in ast.items().items() {
        codegen.compile_module_item(module_item, &interner);
    }

    unsafe {
        LLVMBuildRetVoid(codegen.context.builder);

        let ir = LLVMPrintModuleToString(codegen.context.module);
        if args.verbose {
            println!("{}", std::ffi::CStr::from_ptr(ir).to_string_lossy());
        }
        LLVMDisposeMessage(ir);

        let message = std::ptr::null_mut();
        LLVMVerifyModule(
            codegen.context.module,
            llvm_sys::analysis::LLVMVerifierFailureAction::LLVMPrintMessageAction,
            message,
        );

        let target = CString::new(args.target.unwrap_or_else(|| {
            let target = LLVMGetDefaultTargetTriple();
            let target = std::ffi::CStr::from_ptr(target).to_str().unwrap();
            target.to_string()
        }))
        .unwrap();

        let mut target_triple = std::ptr::null_mut();
        let mut err = std::ptr::null_mut();

        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmPrinters();
        LLVM_InitializeAllAsmParsers();

        if LLVMGetTargetFromTriple(target.as_ptr(), &mut target_triple, &mut err) != 0 {
            return Err(format!(
                "Failed to get target from triple: {}",
                std::ffi::CStr::from_ptr(err).to_string_lossy()
            ));
        }

        let target_machine = LLVMCreateTargetMachine(
            target_triple,
            target.as_ptr(),
            c"generic".as_ptr(),
            c"".as_ptr(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        let output_file = &args.output_file.clone().unwrap_or_else(|| {
            let mut output_file = args.input_file.clone();
            output_file.set_extension("o");
            output_file
        });

        let output_file = output_file.to_str().unwrap();
        let output_file = std::ffi::CString::new(output_file).unwrap();

        let output = std::ptr::null_mut();
        let result = LLVMTargetMachineEmitToFile(
            target_machine,
            codegen.context.module,
            output_file.as_ptr(),
            LLVMCodeGenFileType::LLVMObjectFile,
            output,
        );

        if result != 0 {
            return Err("Failed to emit object file".to_string());
        }

        LLVMDisposeTargetMachine(target_machine);

        if let Some(linker) = args.linker {
            let output_file = &args.output_file.clone().unwrap_or_else(|| {
                let mut output_file = args.input_file.clone();
                output_file.set_extension("out");
                output_file
            });
            let result = std::process::Command::new(linker)
                .arg(output_file)
                .arg("-o")
                .arg(output_file)
                .output()
                .expect("Failed to run linker");

            if !result.status.success() {
                return Err("Failed to link object file".to_string());
            }
        }
    }

    Ok(())
}
