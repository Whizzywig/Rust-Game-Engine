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
		for i in 0..(scene_vertices.len()/3){
			let edge1 = Vertex {position:(scene_vertices[(i*3)+1].position.0-scene_vertices[(i*3)].position.0, scene_vertices[(i*3)+1].position.1-scene_vertices[(i*3)].position.1, scene_vertices[(i*3)+1].position.2-scene_vertices[(i*3)].position.2)};
			let edge2 = Vertex {position:(scene_vertices[(i*3)+2].position.0-scene_vertices[(i*3)].position.0, scene_vertices[(i*3)+2].position.1-scene_vertices[(i*3)].position.1, scene_vertices[(i*3)+2].position.2-scene_vertices[(i*3)].position.2)};
			let delta_uv1 = Texcoord{ coord:(scene_texcoords[(i*3)+1].coord.0+scene_texcoords[(i*3)].coord.0,scene_texcoords[(i*3)+1].coord.1+scene_texcoords[(i*3)].coord.1)};
			let delta_uv2 = Texcoord{ coord:(scene_texcoords[(i*3)+2].coord.0+scene_texcoords[(i*3)].coord.0,scene_texcoords[(i*3)+2].coord.1+scene_texcoords[(i*3)].coord.1)};

			let f = 1.0/(delta_uv1.coord.0*delta_uv2.coord.1- delta_uv2.coord.0*delta_uv1.coord.1);
			//normalize the two normal vectors
			let temp = Tangent{tangent:((f*(delta_uv2.coord.1*edge1.position.0 - delta_uv1.coord.1*edge2.position.0))/255.0,
                                        (f*(delta_uv2.coord.1*edge1.position.1 - delta_uv1.coord.1*edge2.position.1))/255.0,
                                        (f*(delta_uv2.coord.1*edge1.position.2 - delta_uv1.coord.1*edge2.position.2))/255.0)}.normalize();
			scene_tangents_temp.push(temp);
			scene_tangents_temp.push(temp);
			scene_tangents_temp.push(temp);


			let temp = Bitangent{bitangent:((f*(-delta_uv2.coord.0*edge1.position.0 - delta_uv1.coord.0*edge2.position.0))/255.0,
                                            (f*(-delta_uv2.coord.0*edge1.position.1 - delta_uv1.coord.0*edge2.position.1))/255.0,
                                            (f*(-delta_uv2.coord.0*edge1.position.2 - delta_uv1.coord.0*edge2.position.2))/255.0)}.normalize();

			scene_bitangents_temp.push(temp);
			scene_bitangents_temp.push(temp);
			scene_bitangents_temp.push(temp);
		}
		
	}	
}

/// Unstable.
// TODO: shouldn't be just `Two` but `Multi`

pub struct FiveBuffersDefinition<T, U, V, W, X>(pub PhantomData<(T, U, V, W, X)>);

impl<T, U, V, W, X> FiveBuffersDefinition<T, U, V, W, X> {
    #[inline]
    pub fn new() -> FiveBuffersDefinition<T, U, V, W, X>{
        FiveBuffersDefinition(PhantomData)
    }
}

unsafe impl<T, U, V, W, X, I> VertexDefinition<I> for FiveBuffersDefinition<T, U, V, W, X>
    where T: Vertex,
          U: Vertex,
          V: Vertex,
          W: Vertex,
          X: Vertex,
          I: ShaderInterfaceDef
{
    type BuffersIter = VecIntoIter<(u32, usize, InputRate)>;
    type AttribsIter = VecIntoIter<(u32, u32, AttributeInfo)>;

    fn definition(
        &self, interface: &I)
        -> Result<(Self::BuffersIter, Self::AttribsIter), IncompatibleVertexDefinitionError> {
        let attrib = {
            let mut attribs = Vec::with_capacity(interface.elements().len());
            for e in interface.elements() {
                let name = e.name.as_ref().unwrap();

                let (infos, buf_offset) = if let Some(infos) = <T as Vertex>::member(name) {
                    (infos, 0)
                } else if let Some(infos) = <U as Vertex>::member(name) {
                    (infos, 1)
                } else if let Some(infos) = <V as Vertex>::member(name) {
                    (infos, 2)
                } else if let Some(infos) = <W as Vertex>::member(name) {
                    (infos, 3)
                }else if let Some(infos) = <X as Vertex>::member(name) {
                    (infos, 4)
                }else {
                    return Err(IncompatibleVertexDefinitionError::MissingAttribute {
                        attribute: name.clone().into_owned(),
                    });
                };

                if !infos.ty.matches(infos.array_size,
                                     e.format,
                                     e.location.end - e.location.start)
                    {
                        return Err(IncompatibleVertexDefinitionError::FormatMismatch {
                            attribute: name.clone().into_owned(),
                            shader: (e.format, (e.location.end - e.location.start) as usize),
                            definition: (infos.ty, infos.array_size),
                        });
                    }

                let mut offset = infos.offset;
                for loc in e.location.clone() {
                    attribs.push((loc,
                                  buf_offset,
                                  AttributeInfo {
                                      offset: offset,
                                      format: e.format,
                                  }));
                    offset += e.format.size().unwrap();
                }
            }
            attribs
        }.into_iter(); // TODO: meh

        let buffers = vec![
            (0, mem::size_of::<T>(), InputRate::Vertex),
            (1, mem::size_of::<U>(), InputRate::Vertex),
            (2, mem::size_of::<V>(), InputRate::Vertex),
            (3, mem::size_of::<W>(), InputRate::Vertex),
            (4, mem::size_of::<X>(), InputRate::Vertex),
        ].into_iter();

        Ok((buffers, attrib))
    }
}

unsafe impl<T, U, V, W, X> VertexSource<Vec<Arc<BufferAccess + Send + Sync>>> for FiveBuffersDefinition<T, U, V, W, X>
    where T: Vertex,
          U: Vertex,
          V: Vertex,
          W: Vertex,
          X: Vertex
{
    #[inline]
    fn decode(&self, source: Vec<Arc<BufferAccess + Send + Sync>>)
              -> (Vec<Box<BufferAccess + Send + Sync>>, usize, usize) {
        // FIXME: safety
        assert_eq!(source.len(), 5);
        let vertices = [
            source[0].size() / mem::size_of::<T>(),
            source[1].size() / mem::size_of::<U>(),
            source[2].size() / mem::size_of::<V>(),
            source[3].size() / mem::size_of::<W>(),
            source[4].size() / mem::size_of::<X>()
        ].iter()
            .cloned()
            .min()
            .unwrap();
        (vec![Box::new(source[0].clone()), Box::new(source[1].clone()),Box::new(source[2].clone()), Box::new(source[3].clone()), Box::new(source[4].clone())], vertices, 1)
    }
}

unsafe impl<'a, T, U, V, W, X, Bt, Bu, Bv, Bw, Bx> VertexSource<(Bt, Bu, Bv, Bw, Bx)> for FiveBuffersDefinition<T, U, V, W, X>
    where T: Vertex,
          Bt: TypedBufferAccess<Content = [T]> + Send + Sync + 'static,
          U: Vertex,
          Bu: TypedBufferAccess<Content = [U]> + Send + Sync + 'static,
          V: Vertex,
          Bv: TypedBufferAccess<Content = [V]> + Send + Sync + 'static,
          W: Vertex,
          Bw: TypedBufferAccess<Content = [W]> + Send + Sync + 'static,
          X: Vertex,
          Bx: TypedBufferAccess<Content = [X]> + Send + Sync + 'static
{
    #[inline]
    fn decode(&self, source: (Bt, Bu, Bv, Bw, Bx)) -> (Vec<Box<BufferAccess + Send + Sync>>, usize, usize) {
        let vertices = [source.0.len(), source.1.len(), source.2.len(), source.3.len(),source.4.len()]
            .iter()
            .cloned()
            .min()
            .unwrap();
        (vec![Box::new(source.0) as Box<_>, Box::new(source.1) as Box<_>, Box::new(source.2) as Box<_>, Box::new(source.3) as Box<_>, Box::new(source.4) as Box<_>], vertices, 1)
    }
}