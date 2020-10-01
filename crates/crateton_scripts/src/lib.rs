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
use cranelift_module::FuncOrDataId;

use std::path::{Path, PathBuf};
use std::os::raw::{c_char, c_int};
use std::mem;
use glob::glob;


use rustc_interface::{Queries, interface::{Config, Compiler}};
use rustc_data_structures::fx::FxHashMap;
use rustc_target::spec::PanicStrategy;
use rustc_driver::Compilation;

struct Callbacks;

use crateton_core::Foo;

impl rustc_driver::Callbacks for Callbacks {
  fn config(&mut self, config: &mut Config) {
    //config.opts.cg.panic = Some(PanicStrategy::Abort);
    config.opts.maybe_sysroot =
      //Some(PathBuf::from("/Users/will/Code/rustc_codegen_cranelift/build_sysroot/sysroot"));
      Some(PathBuf::from("/Users/will/.rustup/toolchains/nightly-x86_64-apple-darwin"))
  }

  fn after_analysis<'tcx>(&mut self, compiler: &Compiler, queries: &'tcx Queries<'tcx>) -> Compilation {
    queries.global_ctxt().unwrap().peek_mut().enter(|tcx| {
      let mut jit_builder = SimpleJITBuilder::with_isa(
          rustc_codegen_cranelift::build_isa(tcx.sess, false),
          cranelift_module::default_libcall_names(),
      );

      //let imported_symbols =
      //  rustc_codegen_cranelift::driver::jit::load_imported_symbols_for_jit(tcx);
      //jit_builder.symbols(imported_symbols);
     
      //jit_builder.symbols(vec![("callback", callback as *const u8)]);

      let mut jit_module: Module<SimpleJITBackend> = Module::new(jit_builder);
 
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

      let finalized_func = if let FuncOrDataId::Func(func) = jit_module.get_name("entry").unwrap() {
        jit_module.get_finalized_function(func)
      } else {
        unreachable!()
      };

      let entry: extern "C" fn(&mut Foo) -> () = unsafe { mem::transmute(finalized_func) };

      let mut foo = Foo { x: 0, y: 2.0 };
      entry(&mut foo);
      println!("{:?}", foo);
      
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

pub fn setup_scripts() {
  let source =  String::from(r#"
use crateton_core::Foo;

#[no_mangle]
pub fn entry(foo: &mut Foo) {
  println!("YEE HAW {:?}", foo);
  foo.x += 1;
}
"#);

  let cmd = cargo_metadata::MetadataCommand::new();
  let metadata = cmd.exec().unwrap();
  let package = metadata.root_package().unwrap();

  let dep_dir = "target/debug/deps";
  let externs = package.dependencies.iter().map(|d| {
    let rlib = glob(&format!("{}/lib{}-*.rlib", dep_dir, d.name)).unwrap().next().unwrap().unwrap();
    format!("--extern {}={}", d.name, rlib.display())
  }).collect::<Vec<_>>().join(" ");

  rustc_driver::catch_with_exit_code(|| {
    let args = format!("rustc dummy.rs --crate-type lib --edition=2018 -L dependency={} {}", dep_dir, externs);
    let args = args.split(" ").collect::<Vec<_>>();
   
    rustc_driver::run_compiler(
      &args.into_iter().map(|s| String::from(s)).collect::<Vec<_>>(),
      &mut Callbacks,
      Some(Box::new(StringLoader { source })),
      None,
      None
    )
  });
}