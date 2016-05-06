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
extern crate chrono;

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

    vertex Instance {
        translate: [f32; 3] = "a_Translate",
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
        instance: gfx::InstanceBuffer<Instance> = (),
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
const MAX_INSTANCE_COUNT: usize = 1000 * 1000;

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
        Point3::new(111.5f32, -115.0, 113.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), 4.0/3.0, 1.0, 1000.0);

    let proview = proj * view.mat;

    // instance handling

    let instance_buf = factory.create_buffer_dynamic(MAX_INSTANCE_COUNT,
                                                     gfx::BufferRole::Vertex,
                                                     gfx::Bind::empty()).unwrap();

    // data pipeline
    let data = pipe::Data {
        vbuf: vbuf,
        transform: proview.into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color,
        out_depth: main_depth,
        instance: instance_buf,
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

        let y_max = 100;
        let x_max = 100;
        let gap = 0.5;
        for y in 0..y_max {
            for x in 0..x_max {
                w.create_now()
                .with(Pos(Vec3f::new(0.0, 0.0, 0.0)))
                .with(MoveTo(Vec3f::new(
                    ((x as f32) - ((x_max - 1) as f32) / 2.0) * (2.0 + gap),
                    ((y as f32) - ((y_max - 1) as f32) / 2.0) * (2.0 + gap),
                    1.0,
                ), 0.05))
                .build();
            }
        }

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
    main_side.tx.send(factory.create_command_buffer().into()).unwrap();
    main_side.tx.send(factory.create_command_buffer().into()).unwrap();

    struct Render {
        encoder: EncoderChannel,
        data: pipe::Data<gfx_device_gl::Resources>,
        slice: gfx::Slice<gfx_device_gl::Resources>,
        pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
        proview: cgmath::Matrix4<f32>,
    }
    impl specs::System<()> for Render {
        fn run(&mut self, arg: RunArg, _: ()) {
            let (poss, entities) = arg.fetch(|w| {
                (w.read::<Pos>(), w.entities())
            });

            let mut encoder = self.encoder.rx.recv().unwrap();

            encoder.clear(&self.data.out_color, CLEAR_COLOR);
            encoder.clear_depth(&self.data.out_depth, 1.0);

            //println!("Start");
            // Insert a component for each entity in sb
            let mut v = vec![];
            for (_eid, pos) in (&entities, &poss).iter() {
                v.push(Instance { translate: pos.0.into() });
            }
            let instance_count = std::cmp::min(v.len(),
                                               MAX_INSTANCE_COUNT);
            let m = self.proview;

            let locals = Locals { transform: m.into() };
            self.data.transform = m.into();
            self.slice.instances = Some((instance_count as u32, 0));
            encoder.update_constant_buffer(&self.data.locals, &locals);
            encoder.update_buffer(&self.data.instance, &v[..instance_count], 0).unwrap();
            encoder.draw(&self.slice, &self.pso, &self.data);
            //println!("End");


            self.encoder.tx.send(encoder).unwrap();
        }
    }

    struct Mover;
    impl specs::System<()> for Mover {
        fn run(&mut self, arg: RunArg, _: ()) {
            let (mut poss, move_tos, entities) = arg.fetch(|w| {
                (w.write::<Pos>(), w.read::<MoveTo>(), w.entities())
            });

            for (_eid, a, b) in (&entities, &mut poss, &move_tos).iter() {
                use cgmath::InnerSpace;

                //println!("Entity @{:?}", a.0);

                let distance = (a.0 - b.0).magnitude().abs();
                let new_distance = (distance - b.1).max(0.0);
                let f = if distance > 0.0 {
                    new_distance / distance
                } else {
                    0.0
                };

                a.0 = b.0.lerp(a.0, f);
                //println!("-> Entity @{:?}", a.0);
            }
        }
    }

    planner.add_system(Render {
        encoder: render_side,
        data: data,
        slice: slice,
        pso: pso,
        proview: proview,
    }, "render", 10);
    planner.add_system(Mover, "mover", 20);



    struct Fps {
        ms_accum: u32,
        history: Vec<u32>,
    }
    impl Fps {
        fn new() -> Self {
            Fps { ms_accum: 0, history: vec![] }
        }
        fn frame(&mut self, time: u32) {
            self.ms_accum += time;
            self.history.push(time);

            if self.ms_accum >= 1000 {
                let mut min = self.history[0];
                let mut max = self.history[0];
                let mut average = 0;
                let frames = self.history.len();

                for e in &self.history {
                    use std::cmp;
                    min = cmp::min(min, *e);
                    max = cmp::max(max, *e);
                    average += *e;
                }
                let average = (average as f32) / (frames as f32);
                let mut variance = 0.0;
                for e in &self.history {
                    let e = *e as f32;

                    variance += (e - average) * (e - average)
                }

                println!("FPS: {}, min: {} ms, max: {} ms, avg: {} ms, var: {}",
                         frames, min, max, average, variance);

                self.ms_accum = 0;
                self.history.clear();
            }
        }
    }

    let mut fps = Fps::new();

    // main loop
    let mut running = true;
    while running {
        use chrono::Duration;
        let logic_render_time = Duration::span(|| {

            // loop over events
            for event in window.poll_events() {
                match event {
                    glutin::Event::KeyboardInput(
                        _,
                        _,
                        Some(glutin::VirtualKeyCode::Escape)
                    )
                    | glutin::Event::Closed => running = false,
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

            main_side.tx.send(encoder).unwrap();
        }).num_milliseconds();

        fps.frame(logic_render_time as u32);
    }
}
