use boa_ast::Expression;
use boa_ast::ModuleItem;
use boa_ast::Statement;
use boa_interner::Interner;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMLinkage;
use std::ffi::CString;

pub struct LLVMContext {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
    pub root_function_prototype: LLVMTypeRef,
    pub root_function: LLVMValueRef,
    pub entry_block: LLVMBasicBlockRef,
}

impl LLVMContext {
    pub fn new(module_name: &str) -> Self {
        unsafe {
            let module_name = CString::new(module_name).unwrap();

            let context = LLVMContextCreate();
            let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
            let builder = LLVMCreateBuilderInContext(context);

            let mut param_types = vec![];
            let root_function_prototype = LLVMFunctionType(
                LLVMVoidTypeInContext(context),
                param_types.as_mut_ptr(),
                0,
                0,
            );
            let root_function_name = CString::new("main").unwrap();
            let root_function =
                LLVMAddFunction(module, root_function_name.as_ptr(), root_function_prototype);

            let entry_block = {
                let entry = CString::new("entry").unwrap();

                LLVMAppendBasicBlock(root_function, entry.as_ptr())
            };

            LLVMPositionBuilderAtEnd(builder, entry_block);

            LLVMContext {
                context,
                module,
                builder,
                root_function,
                root_function_prototype,
                entry_block,
            }
        }
    }

    pub fn create_string_literal(&self, string: &str) -> LLVMValueRef {
        unsafe {
            let c_string = CString::new(string).unwrap();
            let str = CString::new("str").unwrap();

            LLVMBuildGlobalStringPtr(self.builder, c_string.as_ptr(), str.as_ptr())
        }
    }
}

impl Drop for LLVMContext {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

pub struct CodeGenerator {
    pub context: LLVMContext,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self {
            context: LLVMContext::new("main"),
        }
    }
}

impl CodeGenerator {
    pub fn compile_module_item(
        &mut self,
        module_item: &ModuleItem,
        interner: &Interner,
    ) -> Option<LLVMValueRef> {
        match module_item {
            ModuleItem::ImportDeclaration(_) => todo!(),
            ModuleItem::ExportDeclaration(_) => todo!(),
            ModuleItem::StatementListItem(sli) => match sli {
                boa_ast::StatementListItem::Statement(statement) => {
                    self.compile_statement(statement, interner)
                }
                boa_ast::StatementListItem::Declaration(_declaration) => todo!(),
            },
        }
    }

    pub fn compile_expression(
        &mut self,
        expression: &Expression,
        interner: &Interner,
    ) -> Option<LLVMValueRef> {
        match expression {
            Expression::This => todo!(),
            Expression::Identifier(_) => todo!(),
            Expression::Literal(literal) => match literal {
                boa_ast::expression::literal::Literal::String(string) => {
                    let string_value = interner.resolve_expect(*string).utf8().unwrap();

                    Some(self.context.create_string_literal(string_value))
                }
                boa_ast::expression::literal::Literal::Num(n) => Some(unsafe {
                    LLVMConstReal(LLVMDoubleTypeInContext(self.context.context), *n)
                }),
                boa_ast::expression::literal::Literal::Int(n) => Some(unsafe {
                    LLVMConstInt(LLVMInt32TypeInContext(self.context.context), *n as u64, 0)
                }),
                boa_ast::expression::literal::Literal::BigInt(_) => todo!(),
                boa_ast::expression::literal::Literal::Bool(bool) => Some(unsafe {
                    LLVMConstInt(
                        LLVMInt1TypeInContext(self.context.context),
                        if *bool { 1 } else { 0 },
                        0,
                    )
                }),
                boa_ast::expression::literal::Literal::Null => todo!(),
                boa_ast::expression::literal::Literal::Undefined => todo!(),
            },
            Expression::RegExpLiteral(_) => todo!(),
            Expression::ArrayLiteral(_) => todo!(),
            Expression::ObjectLiteral(_) => todo!(),
            Expression::Spread(_) => todo!(),
            Expression::FunctionExpression(_function) => todo!(),
            Expression::ArrowFunction(_) => todo!(),
            Expression::AsyncArrowFunction(_) => todo!(),
            Expression::GeneratorExpression(_) => todo!(),
            Expression::AsyncFunctionExpression(_) => todo!(),
            Expression::AsyncGeneratorExpression(_) => todo!(),
            Expression::ClassExpression(_) => todo!(),
            Expression::TemplateLiteral(_) => todo!(),
            Expression::PropertyAccess(_) => todo!(),
            Expression::New(_) => todo!(),
            Expression::Call(call) => {
                let identifier = match call.function() {
                    Expression::Identifier(ident) => {
                        interner.resolve_expect(ident.sym()).utf8().unwrap()
                    }
                    i => panic!("Unknown function identifier: {:#?}", i),
                };

                let mut args = vec![];
                for arg in call.args() {
                    args.push(self.compile_expression(arg, interner).unwrap());
                }

                let mut arg_types = vec![];

                for arg in &args {
                    arg_types.push(unsafe { LLVMTypeOf(*arg) });
                }

                let function = unsafe {
                    let c_name = CString::new(identifier).unwrap();
                    let function = LLVMGetNamedFunction(self.context.module, c_name.as_ptr());

                    let function_type = LLVMFunctionType(
                        LLVMInt32TypeInContext(self.context.context),
                        arg_types.as_mut_ptr(),
                        arg_types.len() as u32,
                        0,
                    );

                    if function.is_null() {
                        let function =
                            LLVMAddFunction(self.context.module, c_name.as_ptr(), function_type);

                        LLVMSetLinkage(function, LLVMLinkage::LLVMExternalLinkage);

                        (function, function_type)
                    } else {
                        (function, function_type)
                    }
                };

                Some(unsafe {
                    LLVMBuildCall2(
                        self.context.builder,
                        function.1,
                        function.0,
                        args.as_mut_ptr(),
                        args.len() as u32,
                        b"\0".as_ptr(),
                    )
                })
            }
            Expression::SuperCall(_) => todo!(),
            Expression::ImportCall(_) => todo!(),
            Expression::Optional(_) => todo!(),
            Expression::TaggedTemplate(_) => todo!(),
            Expression::NewTarget => todo!(),
            Expression::ImportMeta => todo!(),
            Expression::Assign(_) => todo!(),
            Expression::Unary(_) => todo!(),
            Expression::Update(_) => todo!(),
            Expression::Binary(_) => todo!(),
            Expression::BinaryInPrivate(_) => todo!(),
            Expression::Conditional(_) => todo!(),
            Expression::Await(_) => todo!(),
            Expression::Yield(_) => todo!(),
            Expression::Parenthesized(_) => todo!(),
            _ => todo!(),
        }
    }

    pub fn compile_statement(
        &mut self,
        statement: &Statement,

        interner: &Interner,
    ) -> Option<LLVMValueRef> {
        match statement {
            boa_ast::Statement::Block(block) => {
                for statement_list_item in block.statement_list().iter() {
                    match statement_list_item {
                        boa_ast::StatementListItem::Statement(statement) => {
                            self.compile_statement(statement, interner);
                        }
                        boa_ast::StatementListItem::Declaration(_) => todo!(),
                    }
                }

                None
            }
            boa_ast::Statement::Var(_) => todo!(),
            boa_ast::Statement::Empty => todo!(),
            boa_ast::Statement::Expression(expression) => {
                self.compile_expression(expression, interner)
            }
            boa_ast::Statement::If(_) => todo!(),
            boa_ast::Statement::DoWhileLoop(_) => todo!(),
            boa_ast::Statement::WhileLoop(while_loop) => unsafe {
                let condition_block =
                    LLVMAppendBasicBlock(self.context.root_function, b"condition\0".as_ptr());
                let body_block =
                    LLVMAppendBasicBlock(self.context.root_function, b"body\0".as_ptr());

                let end_block = LLVMAppendBasicBlock(self.context.root_function, b"end\0".as_ptr());

                LLVMBuildBr(self.context.builder, condition_block);

                LLVMPositionBuilderAtEnd(self.context.builder, condition_block);

                let condition = self
                    .compile_expression(while_loop.condition(), interner)
                    .unwrap();

                LLVMBuildCondBr(self.context.builder, condition, body_block, end_block);

                LLVMPositionBuilderAtEnd(self.context.builder, body_block);

                let body = while_loop.body();

                LLVMPositionBuilderAtEnd(self.context.builder, body_block);

                self.compile_statement(body, interner);

                LLVMBuildBr(self.context.builder, condition_block);

                LLVMPositionBuilderAtEnd(self.context.builder, end_block);

                None
            },
            boa_ast::Statement::ForLoop(_) => todo!(),
            boa_ast::Statement::ForInLoop(_) => todo!(),
            boa_ast::Statement::ForOfLoop(_) => todo!(),
            boa_ast::Statement::Switch(_) => todo!(),
            boa_ast::Statement::Continue(_) => todo!(),
            boa_ast::Statement::Break(_) => todo!(),
            boa_ast::Statement::Return(_) => todo!(),
            boa_ast::Statement::Labelled(_) => todo!(),
            boa_ast::Statement::Throw(_) => todo!(),
            boa_ast::Statement::Try(_) => todo!(),
            boa_ast::Statement::With(_) => todo!(),
        }
    }
}
