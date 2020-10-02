#![feature(clamp)]

pub mod physics;
pub mod controls;

#[derive(Debug)]
pub struct Foo { 
    pub x: i32,
    pub y: f32
}

pub fn bar<T: std::fmt::Debug>(t: T) {
    println!("{:?}", t);
}