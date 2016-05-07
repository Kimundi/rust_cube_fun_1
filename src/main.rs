#[macro_use]
pub extern crate gfx;
pub extern crate gfx_window_glutin;
pub extern crate gfx_device_gl;
pub extern crate glutin;

pub extern crate cgmath;
pub extern crate specs;
pub extern crate chrono;

#[macro_use]
extern crate lazy_static;

mod systems;
mod components;
mod prelude;
mod render;

pub fn main() {
    render::main();
}
