extern crate cgmath;

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::device::Queue;

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
#[allow(dead_code)]
impl Mesh{
	pub fn new(name:String, file:&str, pos: cgmath::Matrix4<f32>, queue:Arc<Queue>, device:Arc<Device>)->Mesh{
		Mesh{
			name: name,
			render_object: crate::support::RenderObject::new(file,queue,device),
			position:pos
		}
	}
	pub fn get_position(self)-> [f32; 3]{
		[self.position.w.x,self.position.w.y,self.position.w.z]
	}
}