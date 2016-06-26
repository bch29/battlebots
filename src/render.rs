use math::*;
use rpc::*;

use render_utils::*;
use glium::{Surface, DrawParameters, DrawError};
use glium::index::PrimitiveType;
use glium::uniforms::{EmptyUniforms, UniformsStorage};
use glium::backend::Facade;

use cgmath::prelude::*;
use cgmath::{Vector3, Matrix3, Matrix4, Quaternion, rad};

use std::sync::{Arc, Mutex};

type Data =
    InstancedData<Vertex, Attr, <GlobalUniforms as IntoUniforms>::IntoUniforms>;

pub struct DrawState {
    body_data: Data,
    radar_data: Data,
    gun_data: Data,

    bots: Arc<Mutex<Vec<BotState>>>,
}

fn mat_3_to_4<S: Copy + Zero + One>(mat: Matrix3<S>) -> Matrix4<S> {
    let cols: [[S; 3]; 3] = mat.into();

    let z = Zero::zero();
    let o = One::one();

    Matrix4::from([[cols[0][0], cols[0][1], cols[0][2], z],
                   [cols[1][0], cols[1][1], cols[1][2], z],
                   [cols[2][0], cols[2][1], cols[2][2], z],
                   [z, z, z, o]])
}

impl DrawState {
    pub fn new<F>(display: &F, bots: Arc<Mutex<Vec<BotState>>>) -> Self
        where F: Facade
    {
        let num_bots = {
            bots.lock().unwrap().len()
        };

        let body = DataBuilder {
            vertices: vec![
                Vertex::new([-0.6, -0.4], [1.0, 0.0, 0.0, 1.0]),
                Vertex::new([-0.6, 0.4], [1.0, 0.0, 0.0, 1.0]),
                Vertex::new([0.6, 0.4], [1.0, 0.0, 0.0, 1.0]),
                Vertex::new([0.6, -0.4], [1.0, 0.0, 0.0, 1.0]),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            uniforms: GlobalUniforms { global_matrix: Matrix4::from_scale(0.1).into() },
            vshader_src: include_str!("bin/vshader.glsl"),
            fshader_src: include_str!("bin/fshader.glsl"),
        };

        let radar = DataBuilder {
            vertices: vec![
                Vertex::new([0.0, 0.0], [0.0, 1.0, 0.0, 1.0]),
                Vertex::new([0.3, 0.3], [0.0, 1.0, 0.0, 1.0]),
                Vertex::new([0.3, -0.3], [0.0, 1.0, 0.0, 1.0]),
            ],
            indices: vec![0, 1, 2],
            ..body.clone()
        };

        let gun = DataBuilder {
            vertices: vec![
                Vertex::new([ 0.0, -0.06,], [0.0, 0.0, 1.0, 1.0]),
                Vertex::new([ 0.0, 0.06, ], [0.0, 0.0, 1.0, 1.0]),
                Vertex::new([ 0.8, 0.06, ], [0.0, 0.0, 1.0, 1.0]),
                Vertex::new([ 0.8, -0.06,], [0.0, 0.0, 1.0, 1.0]),
            ],
            ..body.clone()
        };

        let prim_type = PrimitiveType::TriangleStrip;

        DrawState {
            body_data: body.build_instanced(display, num_bots, prim_type).unwrap(),
            radar_data: radar.build_instanced(display, num_bots, prim_type).unwrap(),
            gun_data: gun.build_instanced(display, num_bots, prim_type).unwrap(),

            bots: bots,
        }
    }

    pub fn update(&mut self) {
        let bots = {
            let bots = self.bots.lock().unwrap();
            bots.clone()
        };

        fn update_one<F>(data: &mut Data, bots: &[BotState], select_heading_pos: F)
            where F: Fn(&BotState) -> (f64, Vector2)
        {

            let iter = bots.iter().map(|bot| {
                let (heading, pos) = select_heading_pos(bot);

                let rot = mat_3_to_4(Matrix3::from(Quaternion::from_angle_z(rad(heading as f32))));
                let transl =
                    Matrix4::from_translation(Vector3::new(pos.x as f32, pos.y as f32, 0.0));

                Attr { instance_matrix: (transl * rot).into() }
            });

            data.update_instances(iter).unwrap();
        }

        update_one(&mut self.body_data, &bots, |bot| (bot.heading, bot.pos));
        update_one(&mut self.radar_data,
                   &bots,
                   |bot| (bot.radar_heading, bot.pos));
        update_one(&mut self.gun_data, &bots, |bot| (bot.gun_heading, bot.pos));
    }

    pub fn draw<S>(&self, surface: &mut S, params: &DrawParameters) -> Result<(), DrawError>
        where S: Surface
    {

        try!(self.body_data.draw(surface, params));
        try!(self.gun_data.draw(surface, params));
        try!(self.radar_data.draw(surface, params));

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Vertex {
            position: position,
            color: color,
        }
    }
}

#[derive(Copy, Clone)]
struct Attr {
    instance_matrix: [[f32; 4]; 4],
}

implement_vertex!(Vertex, position, color);
implement_vertex!(Attr, instance_matrix);

#[derive(Clone)]
struct GlobalUniforms {
    global_matrix: [[f32; 4]; 4],
}

impl IntoUniforms for GlobalUniforms {
    type IntoUniforms = UniformsStorage<'static, [[f32; 4]; 4], EmptyUniforms>;

    fn into_uniforms(self) -> Self::IntoUniforms {
        UniformsStorage::new("global_matrix", self.global_matrix)
    }
}
