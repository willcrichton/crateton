#![feature(clamp)]

pub mod physics;
pub mod controls;

#[derive(Debug)]
pub struct Foo { 
    pub x: i32,
    pub y: f32
}

pub fn foo() -> Foo {
    Foo {
        x: 1,
        y: 3.
    }
}