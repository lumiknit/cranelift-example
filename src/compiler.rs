use cranelift::{
    codegen::{
        control::ControlPlane,
        ir::{types, AbiParam, Function, Signature, UserFuncName},
        isa::{self, CallConv},
        Context,
    },
    prelude::{settings, Block, InstBuilder, IntCC, Value},
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use target_lexicon::Triple;

use crate::expr::{BinOp, Expr};

/// Compiled object buffer, which takes 4 i32 arguments and returns an i32 value
pub struct Fn4I32ToI32 {
    buf: memmap2::Mmap,
}

impl Fn4I32ToI32 {
    /// Invoke the function with the given arguments
    pub fn call(&self, a: i32, b: i32, c: i32, d: i32) -> i32 {
        unsafe {
            let func: unsafe extern "sysv64" fn(i32, i32, i32, i32) -> i32 =
                std::mem::transmute(self.buf.as_ptr());
            func(a, b, c, d)
        }
    }
}

fn build_each_expr(expr: &Expr, builder: &mut FunctionBuilder, block: &Block) -> Value {
    match expr {
        Expr::Num(n) => builder.ins().iconst(types::I32, *n as i64),
        Expr::Input(i) => {
            if *i >= 4 {
                panic!("Current compiler only supports 4 inputs");
            }
            println!("block_params: {:?}", builder.block_params(*block));
            builder.block_params(*block)[*i as usize]
        }
        Expr::BinOp(op, lhs, rhs) => {
            let lhs = build_each_expr(lhs, builder, block);
            let rhs = build_each_expr(rhs, builder, block);
            match op {
                BinOp::Add => builder.ins().iadd(lhs, rhs),
                BinOp::Sub => builder.ins().isub(lhs, rhs),
                BinOp::Mul => builder.ins().imul(lhs, rhs),
                BinOp::Div => builder.ins().sdiv(lhs, rhs),
                BinOp::Eq => builder.ins().icmp(IntCC::Equal, lhs, rhs),
            }
        }
    }
}

fn build_with_expr(expr: &Expr, builder: &mut FunctionBuilder) {
    let block = builder.create_block();
    builder.seal_block(block);
    builder.append_block_params_for_function_params(block);

    builder.switch_to_block(block);

    let result = build_each_expr(expr, builder, &block);

    builder.ins().return_(&[result]);
}

/// Compile the expression into a single function with cranelift
pub fn compile_expr(expr: &Expr) -> Result<Fn4I32ToI32, String> {
    let mut sig = Signature::new(CallConv::SystemV);

    // Add 4 i32 arguments
    for _ in 0..4 {
        sig.params.push(AbiParam::new(types::I32));
    }

    // Return an i32 value
    sig.returns.push(AbiParam::new(types::I32));

    // Create a new function
    let name = UserFuncName::default();
    let mut func = Function::with_name_signature(name, sig);

    // Create a context and builder
    let mut ctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut ctx);

    // Create a new block
    build_with_expr(expr, &mut builder);

    // Finalize the builder
    builder.finalize();

    // Set-up ISA
    let builder = settings::builder();
    let isa = settings::Flags::new(builder);

    let isa = match isa::lookup(Triple::host()) {
        Err(e) => return Err(format!("Failed to look up host triple: {}", e)),
        Ok(isa_builder) => isa_builder.finish(isa),
    }
    .unwrap();

    // Compile
    let mut ctx = Context::for_function(func);
    let mut ctrl_plane = ControlPlane::default();
    let code = ctx.compile(&*isa, &mut ctrl_plane).unwrap();

    // Create a memory map
    let mut buf = memmap2::MmapOptions::new()
        .len(code.code_buffer().len())
        .map_anon()
        .unwrap();
    buf.copy_from_slice(code.code_buffer());

    let buffer = Fn4I32ToI32 {
        buf: buf.make_exec().unwrap(),
    };

    Ok(buffer)
}
