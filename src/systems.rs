use prelude::*;
use components::*;

#[derive(Clone)]
pub struct Context;

/////////////////////////////////////////////////////////////

impl specs::System<Context> for ::render::Render {
    fn run(&mut self, arg: RunArg, _: Context) {
        use render::CLEAR_COLOR;
        use render::MAX_INSTANCE_COUNT;
        use render::Instance;
        use render::Locals;

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
        let instance_count = cmp::min(v.len(), MAX_INSTANCE_COUNT);
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

pub struct Mover;
impl specs::System<Context> for Mover {
    fn run(&mut self, arg: RunArg, _: Context) {
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
