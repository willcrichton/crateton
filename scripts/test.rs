use crateton_core::Foo;

#[derive(Debug)]
struct La { x: i32 }

#[no_mangle]
pub fn entry(foo: &mut Foo) {
  println!("{:?}", foo);
  foo.x += 1;

  println!("{:?}", La { x: 1 });
}

fn main(){}