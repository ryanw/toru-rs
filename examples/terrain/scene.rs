use super::shaders::*;
use mutunga::Color;
use nalgebra as na;
use noise::{NoiseFn, OpenSimplex};
use std::f32::consts::PI;
use std::time;
use toru::{Canvas, FreeCamera, Texture};

const MESH_RES: isize = 16;
const TEXTURE_RES: u32 = 128;

fn create_quad(x: isize, y: isize) -> Vec<TerrainVertex> {
	let step = 1.0 / MESH_RES as f32;
	let x = step * x as f32;
	let y = step * y as f32;
	let z = 0.0;

	vec![
		TerrainVertex {
			position: na::Point3::new(x, y, z),
		},
		TerrainVertex {
			position: na::Point3::new(x + step, y, z),
		},
		TerrainVertex {
			position: na::Point3::new(x + step, y + step, z),
		},
		TerrainVertex {
			position: na::Point3::new(x, y, z),
		},
		TerrainVertex {
			position: na::Point3::new(x + step, y + step, z),
		},
		TerrainVertex {
			position: na::Point3::new(x, y + step, z),
		},
	]
}

pub struct TerrainScene {
	last_tick_at: time::Instant,
	program: TerrainProgram,
	camera: FreeCamera,
	vertices: Vec<TerrainVertex>,
	transform: na::Matrix4<f32>,
}

impl TerrainScene {
	pub fn new() -> Self {
		// Use texture as heightmap
		let mut texture: Texture<Color> = Texture::new(TEXTURE_RES, TEXTURE_RES);
		let scale = 0.05;
		let n = OpenSimplex::new();
		for y in 0..TEXTURE_RES {
			for x in 0..TEXTURE_RES {
				if let Some(pixel) = texture.get_pixel_mut(x, y) {
					let val = n.get([scale * x as f64, scale * y as f64, 0.0]) + 0.5;
					let r = (val * 255.0) as u8;
					*pixel = Color::rgb(r, r, r);
				}
			}
		}

		// Setup some shaders
		let vertex_shader = TerrainVertexShader::new(texture);
		let fragment_shader = TerrainFragmentShader::new();

		let mut vertices = Vec::with_capacity((6 * MESH_RES * MESH_RES) as usize);
		let edge = MESH_RES / 2;
		for y in -edge..edge {
			for x in -edge..edge {
				vertices.append(&mut create_quad(x, y));
			}
		}

		let transform = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 0.2))
			* na::Matrix4::from_euler_angles(-PI / 3.0, 0.0, 0.0)
			* na::Matrix4::new_scaling(2.0);

		TerrainScene {
			last_tick_at: time::Instant::now(),
			program: TerrainProgram::new(vertex_shader, fragment_shader),
			vertices,
			camera: FreeCamera::new(1.0, 1.0),
			transform,
		}
	}
	pub fn update(&mut self) {
		let dt = self.last_tick_at.elapsed().as_secs_f32();
		self.last_tick_at = time::Instant::now();

		self.program.vertex_shader.set_camera(&self.camera);

		self.transform *= na::Matrix4::from_euler_angles(0.0, 0.0, -0.234 * PI * dt);
	}

	pub fn draw(&mut self, canvas: &mut Canvas<Color>) {
		let mut ctx = canvas.context();

		let w = ctx.width() as f32;
		let h = ctx.height() as f32;
		// Update camera size/aspect if the canvas as been resized
		if w != self.camera.width() || h != self.camera.height() {
			self.camera.resize(w, h);
		}
		self.update();

		ctx.clear();
		self.program.vertex_shader.set_model(&self.transform);
		ctx.draw_triangles(&mut self.program, self.vertices.iter());
	}
}
