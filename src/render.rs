use prelude::*;
use gfx;
use gfx_device_gl;
use gfx_window_glutin;
use glutin;
use cgmath;

use components::*;
use systems::*;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type Encoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;

pub struct EncoderChannel {
    pub tx: SyncSender<Encoder>,
    pub rx: Receiver<Encoder>,
}

pub fn encoder_channel() -> (EncoderChannel, EncoderChannel) {
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

lazy_static! {
    static ref DEBUG_CUBE: ([Vertex; 24], [u16; 36]) = (
        [
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
        ],
        [
             0,  1,  2,  2,  3,  0, // top
             4,  5,  6,  6,  7,  4, // bottom
             8,  9, 10, 10, 11,  8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ]
    );
}

pub const CLEAR_COLOR: [f32; 4] = [0.1, 0.2, 0.3, 1.0];
pub const MAX_INSTANCE_COUNT: usize = 1000 * 1000;

/// Render system
pub struct Render {
    pub encoder: EncoderChannel,
    pub data: pipe::Data<gfx_device_gl::Resources>,
    pub slice: gfx::Slice<gfx_device_gl::Resources>,
    pub pso: gfx::PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    pub proview: Mat4f,
}

pub fn main() {
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

    // vertex buffer
    let (vbuf, slice) = factory.create_vertex_buffer_with_slice(
        &DEBUG_CUBE.0[..], &DEBUG_CUBE.1[..]);

    //let texels = [[0x20, 0x60, 0x30, 0x00]];
    let texels = [[0x50, 0x50, 0x50, 0x00]];
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
    let mut planner = {
        let mut w = specs::World::new();

        w.register::<Pos>();
        w.register::<MoveTo>();

        let y_max = 100;
        let x_max = 100;
        let gap = 0.0;
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

        specs::Planner::new(w, 4)
    };

    let (main_side, render_side) = encoder_channel();

    // seed render loop with two encoders
    main_side.tx.send(factory.create_command_buffer().into()).unwrap();
    main_side.tx.send(factory.create_command_buffer().into()).unwrap();

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

    struct CameraController {
        wasdqe: [ElementState; 6],
        pos_delta: Vec2f,
        rot_delta: f32,
        zoom_delta: f32,
        frame_reset: bool,
    }

    impl CameraController {
        fn handle(&mut self, event: &glutin::Event) {
            if self.frame_reset {
                self.frame_reset = false;
                self.zoom_delta = 0.0;
            }

            match *event {
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::W)) => self.wasdqe[0] = s,
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::A)) => self.wasdqe[1] = s,
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::S)) => self.wasdqe[2] = s,
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::D)) => self.wasdqe[3] = s,
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::Q)) => self.wasdqe[4] = s,
                glutin::Event::KeyboardInput(
                    s, _, Some(glutin::VirtualKeyCode::E)) => self.wasdqe[5] = s,
                glutin::Event::MouseWheel(d, _) => match d {
                    glutin::MouseScrollDelta::LineDelta(_, s) | glutin::MouseScrollDelta::PixelDelta(_, s) => {
                        if s > 0.0 {
                            self.zoom_delta = 1.0;
                        } else if s < 0.0 {
                            self.zoom_delta = - 1.0;
                        }
                    }
                },
                _ => return,
            }
        }

        fn update(&mut self) {
            let mut mouse_delta = Vec2f::new(0.0, 0.0);
            let mut mouse_rot = 0.0;

            match self.wasdqe[0] {
                ElementState::Pressed => mouse_delta += Vec2f::new(0.0, 1.0),
                _ => (),
            }
            match self.wasdqe[1] {
                ElementState::Pressed => mouse_delta += Vec2f::new(-1.0, 0.0),
                _ => (),
            }
            match self.wasdqe[2] {
                ElementState::Pressed => mouse_delta += Vec2f::new(0.0, -1.0),
                _ => (),
            }
            match self.wasdqe[3] {
                ElementState::Pressed => mouse_delta += Vec2f::new(1.0, 0.0),
                _ => (),
            }
            match self.wasdqe[4] {
                ElementState::Pressed => mouse_rot += 1.0,
                _ => (),
            }
            match self.wasdqe[5] {
                ElementState::Pressed => mouse_rot += -1.0,
                _ => (),
            }

            if mouse_delta.magnitude() != 0.0 {
                mouse_delta = mouse_delta.normalize();
            }
            self.pos_delta = mouse_delta;
            self.rot_delta = mouse_rot;

            println!("d: {:?} r: {:?} z: {:?}",
                     self.pos_delta, self.rot_delta, self.zoom_delta);

            self.frame_reset = true;
        }
    }

    let mut cam = CameraController {
        wasdqe: [ElementState::Released; 6],
        pos_delta: Vec2f::new(0.0, 0.0),
        rot_delta: 0.0,
        zoom_delta: 0.0,
        frame_reset: true,
    };

    // main loop
    let mut running = true;
    while running {
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
                cam.handle(&event);
            }

            cam.update();

            // logic & render
            planner.dispatch(Context);
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
