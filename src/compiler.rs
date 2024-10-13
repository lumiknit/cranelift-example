use cranelift::{
    codegen::ir::{types, AbiParam, FuncRef, Function, UserFuncName},
    prelude::{Block, InstBuilder, IntCC, Value},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};

use crate::{
    expr::{BinOp, Expr},
    runtime,
};

type FuncMap = std::collections::HashMap<String, FuncRef>;

/// Compiled object buffer, which takes 4 i32 arguments and returns an i32 value
pub struct Fn4I32ToI32 {
    buf: *const u8,
}

impl Fn4I32ToI32 {
    /// Invoke the function with the given arguments
    pub fn call(&self, a: i32, b: i32, c: i32, d: i32) -> i32 {
        unsafe {
            let func: unsafe fn(i32, i32, i32, i32) -> i32 = std::mem::transmute(self.buf);
            func(a, b, c, d)
        }
    }
}

/// Insert the expression into the block
/// Since current language has no branch/blocks, this simple function is enough
/// to translate the expression into cranelift IR.
/// However, if you want to support more complex language, you may need to
/// implement more precise state management.
fn insert_expr_to_block(
    expr: &Expr,
    builder: &mut FunctionBuilder,
    block: &Block,
    func_map: &FuncMap,
) -> Value {
    match expr {
        Expr::Num(n) => builder.ins().iconst(types::I32, *n as i64),
        Expr::Input(i) => {
            if *i >= 4 {
                panic!("Current compiler only supports 4 inputs");
            }
            builder.block_params(*block)[*i as usize]
        }
        Expr::BinOp(op, lhs, rhs) => {
            let lhs = insert_expr_to_block(lhs, builder, block, func_map);
            let rhs = insert_expr_to_block(rhs, builder, block, func_map);
            match op {
                BinOp::Add => builder.ins().iadd(lhs, rhs),
                BinOp::Sub => builder.ins().isub(lhs, rhs),
                BinOp::Mul => builder.ins().imul(lhs, rhs),
                BinOp::Div => builder.ins().sdiv(lhs, rhs),
                BinOp::Eq => builder.ins().icmp(IntCC::Equal, lhs, rhs),
            }
        }
        Expr::Call(func, arg) => {
            // For function call, find the symbol first.
            let r = func_map.get(func.to_string()).unwrap();
            let a = insert_expr_to_block(arg, builder, block, func_map);
            let call = builder.ins().call(*r, &[a]);
            builder.inst_results(call)[0]
        }
    }
}

/// Build the function with the given expression
fn build_function_with_expr(expr: &Expr, builder: &mut FunctionBuilder, func_map: &FuncMap) {
    let block = builder.create_block();
    builder.append_block_params_for_function_params(block);
    builder.switch_to_block(block);
    builder.seal_block(block);

    let result = insert_expr_to_block(expr, builder, &block, func_map);

    builder.ins().return_(&[result]);
}

/// Declare runtime functions into the given module.
fn declare_runtime_functions(module: &mut JITModule, func: &mut Function) -> FuncMap {
    let mut func_map = FuncMap::new();

    {
        // Print
        let mut print_sig = module.make_signature();
        print_sig.params.push(AbiParam::new(types::I32));
        print_sig.returns.push(AbiParam::new(types::I32));
        let id = module
            .declare_function("print", Linkage::Import, &print_sig)
            .unwrap();
        let func_ref = module.declare_func_in_func(id, func);
        func_map.insert("print".to_string(), func_ref);
    }

    {
        // Rand
        let mut rand_sig = module.make_signature();
        rand_sig.params.push(AbiParam::new(types::I32));
        rand_sig.returns.push(AbiParam::new(types::I32));
        let id = module
            .declare_function("rand", Linkage::Import, &rand_sig)
            .unwrap();
        let func_ref = module.declare_func_in_func(id, func);
        func_map.insert("rand".to_string(), func_ref);
    }

    func_map
}

/// Compile the expression into a single function with cranelift
pub fn compile_expr(expr: &Expr) -> Result<Fn4I32ToI32, String> {
    // Create JITBuilder and push some runtime functions as symbol
    let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
    builder.symbol("print", runtime::print as *const u8);
    builder.symbol("rand", runtime::rand as *const u8);

    // Create JIT Module
    let mut module = JITModule::new(builder);

    // Now, start to compile the main program.
    // Create a context and set-up function signature
    let mut ctx = module.make_context();
    for _ in 0..4 {
        ctx.func.signature.params.push(AbiParam::new(types::I32));
    }
    ctx.func.signature.returns.push(AbiParam::new(types::I32));
    ctx.func.name = UserFuncName::default();

    // Create a function context
    let mut func_ctx = FunctionBuilderContext::new();

    // Before building the function, declare runtime functions
    let func_map = declare_runtime_functions(&mut module, &mut ctx.func);

    // Build the function
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
    build_function_with_expr(expr, &mut builder, &func_map);
    builder.finalize();

    // Declare and define the main function into the module
    let main_fn = module
        .declare_function("main", Linkage::Local, &ctx.func.signature)
        .unwrap();
    module.define_function(main_fn, &mut ctx).unwrap();
    module.clear_context(&mut ctx);

    // Finalize the definitions
    module
        .finalize_definitions()
        .expect("Failed to finalize definitions");

    // Get the final code
    let code = module.get_finalized_function(main_fn);
    Ok(Fn4I32ToI32 { buf: code })
}
