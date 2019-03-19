use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::vec::IntoIter as VecIntoIter;
use std::path::Path;

extern crate vulkano;
extern crate tobj;

use vulkano::buffer::BufferAccess;
use vulkano::buffer::TypedBufferAccess;
use vulkano::pipeline::shader::ShaderInterfaceDef;
use vulkano::pipeline::vertex::AttributeInfo;
use vulkano::pipeline::vertex::IncompatibleVertexDefinitionError;
use vulkano::pipeline::vertex::InputRate;
use vulkano::pipeline::vertex::Vertex;
use vulkano::pipeline::vertex::VertexDefinition;
use vulkano::pipeline::vertex::VertexSource;
use vulkano::image::immutable::ImmutableImage;
use vulkano::memory::pool::PotentialDedicatedAllocation;
use vulkano::memory::pool::StdMemoryPoolAlloc;
use vulkano::format::R8G8B8A8Srgb;

use crate::math::Tangent;
use crate::math::Bitangent;
use crate::math::Normal;
use crate::math::Texcoord;
use crate::math::Vertex;

pub mod camera;
pub mod object;

#[allow(dead_code)]
pub struct RenderObject{
	pub vertex_buffer:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Vertex]>>,
	pub tangent_buffer:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Tangent]>>,
	pub bitangent_buffer:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Bitnagent]>>,
	pub normal_buffer:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Normal]>>,
	pub texture_base:Arc<ImmutableImage<R8G8B8A8Srgb, PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>,
	pub texture_normal:Arc<ImmutableImage<R8G8B8A8Srgb, PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>,
	pub texture_coords:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[Texcoord]>>,
	pub index_buffer:Arc<vulkano::buffer::cpu_access::CpuAccessibleBuffer<[u32]>>,
}
#[allow(dead_code)]
impl RenderObject{
	fn new(filepath: &str, queue:Arc<vulkano::device::Queue>, device:Arc<vulkano::device::Device>)->RenderObject{
		let mut vertices:Vec<Vertex> = Vec::new();
		let mut tangents_temp:Vec<Tangent> = Vec::new();
		let mut tangents:Vec<Tangent> = Vec::new();
		let mut bitangents_temp:Vec<Bitangent> = Vec::new();
		let mut bitangents:Vec<Bitangent> = Vec::new();
		let mut coords:Vec<Texcoord> = Vec::new();
		let mut normals:Vec<Normal> = Vec::new();
		let mut indices = Vec::new();
		
		let object = tobj::load_obj(&Path::new(&(filepath.clone().to_owned() + ".obj")));
		let (models, _materials) = object.unwrap();
		
		let m = &models[0];
		let mesh = &m.mesh;
		for i in 0..mesh.indices.len(){
			indices.push(mesh.indices[i]);
		}
		for i in 0..(mesh.positions.len()/3){
			verticies.push(Vetrex{ position:(mesh.positions[i*3],
					mesh.positions[(i*3)+1],
					mesh.positions[(i*3)+2])});
			normals.push(Normal{ normal: (mesh.normals[i*3],
					mesh.normals[(i*3)+1],
					mesh.normals[(i*3)+2])});
			coords.push(Texcoord { coord: (mesh.texcoords[i*2],
					1.0 - mesh.texcoords[(i*2)+1])});
		}
		//TODO:bitangent calculations here
		
	}	
}