use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::{IntPredicate, OptimizationLevel};

use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::BasicMetadataValueEnum;
use std::error::Error;
use std::path::PathBuf;

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u32, u32, u32) -> u32;
type FiboFunc = unsafe extern "C" fn(u32) -> u32;

struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,
}

impl<'ctx> CodeGen<'ctx> {
    fn jit_compile_sum(&self) /*-> JitFunction<SumFunc>*/ {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into(), i32_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        function.set_gc("shadow-stack");
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0).unwrap().into_int_value();
        let y = function.get_nth_param(1).unwrap().into_int_value();
        let z = function.get_nth_param(2).unwrap().into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum").unwrap();
        let sum = self.builder.build_int_add(sum, z, "sum").unwrap();

        self.builder.build_return(Some(&sum)).unwrap();

        // let f: JitFunction<SumFunc> = unsafe { self.execution_engine.get_function("sum").unwrap() };
        // ()
    }

    fn jit_compile_fibo(&self) /*-> JitFunction<FiboFunc>*/ {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[i32_type.into()], false);
        let function = self.module.add_function("fibo", fn_type, None);
        function.set_gc("shadow-stack");

        let n = function.get_nth_param(0).unwrap().into_int_value();

        let entry = self.context.append_basic_block(function, "entry");
        let n_is_0 = self.context.append_basic_block(function, "n_is_0");
        let n_is_not_0 = self.context.append_basic_block(function, "n_is_not_0");
        let n_is_1 = self.context.append_basic_block(function, "n_is_1");
        let n_gt_1 = self.context.append_basic_block(function, "n_gt_1");

        self.builder.position_at_end(entry);

        let is_0 = self
            .builder
            .build_int_compare(IntPredicate::EQ, n, i32_type.const_int(0, false), "n == 0")
            .unwrap();
        self.builder
            .build_conditional_branch(is_0, n_is_0, n_is_not_0)
            .unwrap();
        self.builder.position_at_end(n_is_0);
        self.builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();

        self.builder.position_at_end(n_is_not_0);
        let is_1 = self
            .builder
            .build_int_compare(IntPredicate::EQ, n, i32_type.const_int(1, false), "n == 1")
            .unwrap();
        self.builder
            .build_conditional_branch(is_1, n_is_1, n_gt_1)
            .unwrap();
        self.builder.position_at_end(n_is_1);
        self.builder
            .build_return(Some(&i32_type.const_int(1, false)))
            .unwrap();

        self.builder.position_at_end(n_gt_1);

        let n_min_1 = self
            .builder
            .build_int_sub(n, i32_type.const_int(1, false), "n - 1")
            .unwrap();
        let n_min_2 = self
            .builder
            .build_int_sub(n, i32_type.const_int(2, false), "n - 2")
            .unwrap();

        let n_prev1 = self
            .builder
            .build_call(
                function,
                &[BasicMetadataValueEnum::IntValue(n_min_1)],
                "f(n-1)",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let n_prev2 = self
            .builder
            .build_call(
                function,
                &[BasicMetadataValueEnum::IntValue(n_min_2)],
                "f(n-2)",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left
            .into_int_value();
        let n_next = self
            .builder
            .build_int_add(n_prev1, n_prev2, "f(n-1)+f(n-2)")
            .unwrap();

        self.builder.build_return(Some(&n_next)).unwrap();
        //
        // let f: JitFunction<SumFunc> = unsafe { self.execution_engine.get_function("fibo").unwrap() };
        // // unsafe { self.execution_engine.get_function("fibo").unwrap() }
        // ()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = Context::create();
    let module = context.create_module("main-module");

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

    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    module.set_triple(&target_triple);

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
    let codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        execution_engine,
    };

    codegen.jit_compile_fibo();
    codegen.jit_compile_sum();

    println!(
        "\n==== LLVM IR:\n{}",
        codegen.module.print_to_string().to_string()
    );

    let asm = target_machine
        .write_to_memory_buffer(&codegen.module, FileType::Assembly)
        .unwrap();
    println!("\n==== ASM:\n{}", String::from_utf8_lossy(asm.as_slice()));

    let fibo: JitFunction<FiboFunc> = unsafe { codegen.execution_engine.get_function("fibo").unwrap() };
    let sum: JitFunction<SumFunc> = unsafe { codegen.execution_engine.get_function("sum").unwrap() };

    let x = 1u32;
    let y = 2u32;
    let z = 3u32;

    println!("JIT:");
    unsafe {
        println!("sum({}, {}, {}) = {}", x, y, z, sum.call(x, y, z));
        assert_eq!(sum.call(x, y, z), x + y + z);
    }

    let n = 10u32;
    unsafe {
        println!("fibo({}) = {}", n, fibo.call(n));
        assert_eq!(fibo.call(n), 55);
    }

    target_machine
        .write_to_file(
            &codegen.module,
            FileType::Object,
            &PathBuf::new().join("target").join("sum.o"),
        )
        .unwrap();

    Ok(())
}
