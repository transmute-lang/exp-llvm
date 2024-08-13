use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::OptimizationLevel;

use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use std::error::Error;
use std::path::PathBuf;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u32, u32, u32) -> u32;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum").unwrap();
        let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

        self.builder.build_return(Some(&sum)).unwrap();

        unsafe { self.execution_engine.get_function("sum").ok() }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("main-module");

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    let sum = codegen
        .jit_compile_sum()
        .ok_or("Unable to JIT compile `sum`")?;

    let x = 1u32;
    let y = 2u32;
    let z = 3u32;

    println!("JIT:");
    unsafe {
        println!("{} + {} + {} = {}", x, y, z, sum.call(x, y, z));
        assert_eq!(sum.call(x, y, z), x + y + z);
    }

    println!("\nLLVM IR:\n{}", codegen.module.print_to_string().to_string());

    Target::initialize_x86(&InitializationConfig::default());
    let target_triple = TargetMachine::get_default_triple();
    println!("\nTarget triple: {}", target_triple.to_string());
    let target = Target::from_triple(&target_triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    codegen
        .module
        .set_data_layout(&target_machine.get_target_data().get_data_layout());
    codegen.module.set_triple(&target_triple);

    let asm = target_machine
        .write_to_memory_buffer(&codegen.module, FileType::Assembly)
        .unwrap();
    println!("\nASM:\n{}", String::from_utf8_lossy(asm.as_slice()));

    target_machine
        .write_to_file(
            &codegen.module,
            FileType::Object,
            &PathBuf::new().join("target").join("sum.o"),
        )
        .unwrap();

    Ok(())
}
