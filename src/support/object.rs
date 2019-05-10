extern crate cgmath;

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::device::Queue;
use crate::support::camera::Camera;

//TODO: Move all of this into entities and components
pub struct Scene{
	name: String,
	camera: Camera,
	meshes: Arc<Vec<Mesh>>,
	staic_mesh: Arc<Vec<StaticMesh>>,
}
#[allow(dead_code)]
pub struct StaticMesh{
	pub render_object: crate::support::RenderObject,
	pub position: cgmath::Matrix4<f32>,
}
#[allow(dead_code)]
pub struct Mesh{
	name: String,
	pub render_object: crate::support::RenderObject,
	#[derive(Copy, Clone)]
	pub position: cgmath::Matrix4<f32>,
}
#[allow(dead_code)]
impl StaticMesh{
	pub fn new(file: &str , pos: cgmath::Matrix4<f32>, queue:Arc<Queue>, device:Arc<Device>)-> StaticMesh{
		StaticMesh{
			render_object: crate::support::RenderObject::new(file, queue, device),
			position: pos
		}
	}
}
use vulkano::image::immutable::ImmutableImage;
use vulkano::memory::pool::PotentialDedicatedAllocation;
use vulkano::memory::pool::StdMemoryPoolAlloc;
use vulkano::format::R8G8B8A8Srgb;
#[allow(dead_code)]
impl Mesh{
	pub fn new(name:String, file:&str, pos: cgmath::Matrix4<f32>, queue:Arc<Queue>, device:Arc<Device>)->Mesh{
		Mesh{
			name: name,
			render_object: crate::support::RenderObject::new(file,queue,device),
			position:pos
		}
	}
	pub fn new_from_bin(name:string, pos: cgmath::Matrix4<f32>, textures: Arc<[Arc<ImmutableImage<R8G8B8A8Srgb, PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>]>,queue:Arc<Queue>, device:Arc<Device>)->Mesh{
		Mesh{
			name: name,
			render_object:crate::support::RenderObject::new(textures,queue,device),
			position:pos
		}
	}
	pub fn get_position(self)-> [f32; 3]{
		[self.position.w.x,self.position.w.y,self.position.w.z]
	}
}