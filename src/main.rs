// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;
extern crate glutin;
extern crate cgmath;
extern crate specs;

use specs::Join;

use gfx::traits::FactoryExt;
use gfx::Factory;
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

// Declare the vertex format suitable for drawing,
// as well as the constants used by the shaders
// and the pipeline state object format.
// Notice the use of FixedPoint.
gfx_defines!{
    vertex Vertex {
        pos: [i8; 4] = "a_Pos",
        tex_coord: [i8; 2] = "a_TexCoord",
    }

    constant Locals {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0], p[1], p[2], 1],
            tex_coord: t,
        }
    }
}

const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];

pub fn main() {
    use cgmath::{Point3, Vector3};
    use cgmath::{Transform, AffineMatrix3};
    use gfx::traits::FactoryExt;

    // Window builder
    let builder = glutin::WindowBuilder::new()
        .with_title("Triangle example [windowed]".to_string())
        .with_dimensions(1024, 768)
        .with_vsync();

    // Window init
    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

    // shader pipeline
    let pso = factory.create_pipeline_simple(
        include_bytes!("shader/cube_150.glslv"),
        include_bytes!("shader/cube_150.glslf"),
        pipe::new()
    ).unwrap();

    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1, -1,  1], [0, 0]),
        Vertex::new([ 1, -1,  1], [1, 0]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        Vertex::new([-1,  1,  1], [0, 1]),
        // bottom (0, 0, -1)
        Vertex::new([-1,  1, -1], [1, 0]),
        Vertex::new([ 1,  1, -1], [0, 0]),
        Vertex::new([ 1, -1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        Vertex::new([ 1, -1, -1], [0, 0]),
        Vertex::new([ 1,  1, -1], [1, 0]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        Vertex::new([ 1, -1,  1], [0, 1]),
        // left (-1, 0, 0)
        Vertex::new([-1, -1,  1], [1, 0]),
        Vertex::new([-1,  1,  1], [0, 0]),
        Vertex::new([-1,  1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        Vertex::new([ 1,  1, -1], [1, 0]),
        Vertex::new([-1,  1, -1], [0, 0]),
        Vertex::new([-1,  1,  1], [0, 1]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        // back (0, -1, 0)
        Vertex::new([ 1, -1,  1], [0, 0]),
        Vertex::new([-1, -1,  1], [1, 0]),
        Vertex::new([-1, -1, -1], [1, 1]),
        Vertex::new([ 1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0,  1,  2,  2,  3,  0, // top
        4,  5,  6,  6,  7,  4, // bottom
        8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    // vertex buffer
    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

    let texels = [[0x20, 0x60, 0x30, 0x00]];
    let (_, texture_view) = factory.create_texture_const::<gfx::format::Rgba8>(
        gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&texels]
        ).unwrap();

    let sinfo = gfx::tex::SamplerInfo::new(
        gfx::tex::FilterMethod::Bilinear,
        gfx::tex::WrapMode::Clamp);


    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), 4.0/3.0, 1.0, 10.0);

    // data pipeline
    let data = pipe::Data {
        vbuf: vbuf,
        transform: (proj * view.mat).into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color,
        out_depth: main_depth,
    };

    // ECS

    type Vec3f = cgmath::Vector3<f32>;

    #[derive(Clone, Debug)]
    struct Pos(Vec3f);
    impl specs::Component for Pos {
        type Storage = specs::VecStorage<Pos>;
    }

    #[derive(Clone, Debug)]
    struct MoveTo(Vec3f, f32);
    impl specs::Component for MoveTo {
        type Storage = specs::VecStorage<MoveTo>;
    }

    let mut planner = {
        let mut w = specs::World::new();

        w.register::<Pos>();
        w.register::<MoveTo>();

        w.create_now()
            .with(Pos(Vec3f::new(1.0, 1.0, 1.0)))
            .with(MoveTo(Vec3f::new(0.0, 0.0, 0.0), 0.1))
            .build();

        w.create_now()
            .with(Pos(Vec3f::new(0.0, 5.0, 0.0)))
            .with(MoveTo(Vec3f::new(0.0, 0.0, 0.0), 0.1))
            .build();

        specs::Planner::<()>::new(w, 4)
    };

    use specs::RunArg;

    use std::sync::mpsc::SyncSender;
    use std::sync::mpsc::Receiver;
    use std::sync::mpsc::sync_channel;

    type Encoder = gfx::Encoder<
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer>;

    struct EncoderChannel {
        tx: SyncSender<Encoder>,
        rx: Receiver<Encoder>,
    }

    fn encoder_channel() -> (EncoderChannel, EncoderChannel) {
        let (tx1, rx1) = sync_channel(2);
        let (tx2, rx2) = sync_channel(2);
        (EncoderChannel {
            tx: tx1,
            rx: rx2,
        }, EncoderChannel {
            tx: tx2,
            rx: rx1,
        })
    }

    let (main_side, render_side) = encoder_channel();

    // seed render loop with two encoders
    main_side.tx.send(factory.create_command_buffer().into());
    main_side.tx.send(factory.create_command_buffer().into());

    struct Render {
        encoder: EncoderChannel,
        data: pipe::Data<gfx_device_gl::Resources>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
        pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    }
    impl specs::System<()> for Render {
        fn run(&mut self, arg: RunArg, _: ()) {
            let (poss, entities) = arg.fetch(|w| {
                (w.read::<Pos>(), w.entities())
            });

            let mut encoder = self.encoder.rx.recv().unwrap();

            let locals = Locals { transform: self.data.transform };
            encoder.clear(&self.data.out_color, CLEAR_COLOR);
            encoder.clear_depth(&self.data.out_depth, 1.0);


            println!("Start");
            // Insert a component for each entity in sb
            for (eid, pos) in (&entities, &poss).iter() {
                println!("Render {:?} {:?}", eid, pos);
            }
            println!("End");


            encoder.update_constant_buffer(&self.data.locals, &locals);
            encoder.draw(&self.slice, &self.pso, &self.data);

            self.encoder.tx.send(encoder);
        }
    }

    struct Mover;
    impl specs::System<()> for Mover {
        fn run(&mut self, arg: RunArg, _: ()) {
            let (mut poss, move_tos, entities) = arg.fetch(|w| {
                (w.write::<Pos>(), w.read::<MoveTo>(), w.entities())
            });

            for (eid, a, b) in (&entities, &mut poss, &move_tos).iter() {
                use cgmath::InnerSpace;

                println!("Entity @{:?}", a.0);

                let distance = (a.0 - b.0).magnitude().abs();
                let new_distance = (distance - b.1).max(0.0);
                let f = if distance > 0.0 {
                    new_distance / distance
                } else {
                    0.0
                };

                a.0 = b.0.lerp(a.0, f);
                println!("-> Entity @{:?}", a.0);
            }
        }
    }

    planner.add_system(Render {
        encoder: render_side,
        data: data,
        slice: slice,
        pso: pso,
    }, "render", 10);
    planner.add_system(Mover, "mover", 20);

    // main loop
    'main: loop {
        // loop over events
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(
                    _,
                    _,
                    Some(glutin::VirtualKeyCode::Escape)
                )
                | glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        // logic & render
        planner.dispatch(());
        planner.wait();

        let mut encoder = main_side.rx.recv().unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        main_side.tx.send(encoder);
    }
}
