#![feature(rustc_private)]

extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_data_structures;
extern crate rustc_span;
extern crate rustc_errors;
extern crate rustc_error_codes;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_codegen_ssa;
extern crate rustc_target;
extern crate rustc_driver;

use rustc_codegen_cranelift::prelude::*;
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};

use std::path::{Path, PathBuf};
use std::os::raw::{c_char, c_int};

use rustc_interface::{Queries, interface::{Config, Compiler}};
use rustc_data_structures::fx::FxHashMap;
use rustc_target::spec::PanicStrategy;
use rustc_driver::Compilation;

struct Callbacks;

impl rustc_driver::Callbacks for Callbacks {
  fn config(&mut self, config: &mut Config) {
    config.opts.cg.panic = Some(PanicStrategy::Abort);
    config.opts.maybe_sysroot =
      Some(PathBuf::from("/Users/will/Code/rustc_codegen_cranelift/build_sysroot/sysroot"));
  }

  fn after_analysis<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
    queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
      let imported_symbols =
        rustc_codegen_cranelift::driver::jit::load_imported_symbols_for_jit(tcx);

      let mut jit_builder = SimpleJITBuilder::with_isa(
          rustc_codegen_cranelift::build_isa(tcx.sess, false),
          cranelift_module::default_libcall_names(),
      );
      jit_builder.symbols(imported_symbols);
      let mut jit_module: Module<SimpleJITBackend> = Module::new(jit_builder);

      let sig = Signature {
          params: vec![
              AbiParam::new(jit_module.target_config().pointer_type()),
              AbiParam::new(jit_module.target_config().pointer_type()),
          ],
          returns: vec![AbiParam::new(
              jit_module.target_config().pointer_type(), /*isize*/
          )],
          call_conv: CallConv::triple_default(&rustc_codegen_cranelift::target_triple(tcx.sess)),
      };
      let main_func_id = jit_module
          .declare_function("main", Linkage::Import, &sig)
          .unwrap();

      if !tcx.sess.opts.output_types.should_codegen() {
          tcx.sess.fatal("JIT mode doesn't work with `cargo check`.");
      }

      let (_, cgus) = tcx.collect_and_partition_mono_items(LOCAL_CRATE);
      let mono_items = cgus
          .iter()
          .map(|cgu| cgu.items_in_deterministic_order(tcx).into_iter())
          .flatten()
          .collect::<FxHashMap<_, (_, _)>>()
          .into_iter()
          .collect::<Vec<(_, (_, _))>>();

      let mut cx =
        rustc_codegen_cranelift::CodegenCx::new(tcx, jit_module, false);

      rustc_codegen_cranelift::driver::codegen_mono_items(&mut cx, mono_items);
      let (mut jit_module,
           _global_asm, _debug,
           mut unwind_context) =
        cx.finalize();

      rustc_codegen_cranelift::main_shim::maybe_create_entry_wrapper(
        tcx, &mut jit_module, &mut unwind_context, true);
      rustc_codegen_cranelift::allocator::codegen(
        tcx, &mut jit_module, &mut unwind_context);

      jit_module.finalize_definitions();

      let _unwind_register_guard =
        unsafe { unwind_context.register_jit(&mut jit_module) };

      tcx.sess.abort_if_errors();

      let finalized_main: *const u8 = jit_module.get_finalized_function(main_func_id);

      let f: extern "C" fn(c_int, *const *const c_char) -> c_int =
          unsafe { ::std::mem::transmute(finalized_main) };

      let ret = f(0, std::ptr::null());

      jit_module.finish();
    });

    Compilation::Stop
  }
}

struct StringLoader { source: String }

impl rustc_span::source_map::FileLoader for StringLoader {
  fn file_exists(&self, _path: &Path) -> bool { true }
  fn read_file(&self, _path: &Path) -> std::io::Result<String> {
    Ok(self.source.clone())
  }
}

use std::process::Command;

pub fn setup_scripts() {
  let source =  String::from(r#"
use crateton_core::foo;
fn main() {
  println!("{:?}", foo());
}
"#);

  // let output = Command::new("cargo")
  //   .args("build --release -Z unstable-options --build-plan".split(" ").collect::<Vec<_>>())
  //   .output()
  //   .unwrap()
  //   .stdout;
  // let output = String::from_utf8(output).unwrap();
  // let json: serde_json::Value = serde_json::from_str(&output).unwrap();
  // let invocations = json["invocations"].as_array().unwrap();
  // let build_plan = invocations.iter().find(|v| v["package_name"] == "crateton").unwrap();
  // let build_args = build_plan["args"]
  //   .as_array().unwrap()
  //   .iter().map(|v| v.as_str().unwrap()).collect::<Vec<_>>();

  // let extern_crates = build_args.iter().enumerate()
  //   .filter(|(i, arg)| **arg == "--extern")
  //   .map(|(i, _)| &build_args[i..=i+1])
  //   .flatten();

  rustc_driver::init_rustc_env_logger();
  rustc_driver::install_ice_hook();
  rustc_driver::catch_with_exit_code(|| {
    let mut args = vec![
      "rustc", "dummy.rs",
      "--edition=2018",
      "-Cprefer-dynamic"
    ];
    args.extend(extern_crates);
    
    println!("{:?}", args);

    rustc_driver::run_compiler(
      &args.into_iter().map(|s| String::from(s)).collect::<Vec<_>>(),
      &mut Callbacks,
      Some(Box::new(StringLoader { source })),
      None,
      None
    )
  });
}