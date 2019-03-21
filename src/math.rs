extern crate vulkano;
extern crate cgmath;

use std::ops::AddAssign;
use std::ops::Div;

#[derive(Copy, Clone)]
pub struct Vertex {
	pub position: (f32, f32, f32)
}
impl_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub struct Normal {
	pub normal: (f32, f32, f32)
}
impl_vertex!(Normal, normal);

#[derive(Copy, Clone)]
pub struct Texcoord {
	pub coord: (f32, f32)
}
impl_vertex!(Texcoord, coord);

#[derive(Copy, Clone)]
pub struct Tangent {
	pub tangent: (f32, f32, f32)
}
impl_vertex!(Tangent, tangent);

#[derive(Copy, Clone)]
pub struct Bitangent {
	pub bitangent: (f32, f32, f32)
}
impl_vertex!(Bitangent, bitangent);

impl Tangent {
	pub fn normalize(mut self) -> Tangent{
		let length = ((self.tangent.0 * self.tangent.0)+(self.tangent.1 * self.tangent.1)+(self.tangent.2 * self.tangent.2)).sqrt();
		self = self/length;
		return self;
	}
}
impl Bitangent {
	pub fn normalize(mut self) -> Bitangent{
		let length = ((self.bitangent.0 * self.bitangent.0)+(self.bitangent.1 * self.bitangent.1)+(self.bitangent.2 * self.bitangent.2)).sqrt();
		self = self/length;
		return self;
	}
}

impl AddAssign for Bitangent {
	fn add_assign(&mut self, rhs :Self){
		*self = Bitangent{ bitangent:((self.bitangent.0 + rhs.bitangent.0),
		(self.bitangent.1 + rhs.bitangent.1),
		(self.bitangent.2 + rhs.bitangent.2))};
	}
}
impl AddAssign for Tangent {
	fn add_assign(&mut self, rhs :Self){
		*self = Tangent{ tangent:((self.tangent.0 + rhs.tangent.0),
		(self.tangent.1 + rhs.tangent.1),
		(self.tangent.2 + rhs.tangent.2))};
	}
}

impl Div for Bitangent {
	type Output = Bitangent;
	fn div(self, rhs :Bitangent) -> Bitangent {
		Bitangent{ bitangent:((self.bitangent.0 / rhs.bitangent.0),
		(self.bitangent.1 / rhs.bitangent.1),(self.bitangent.2 / rhs.bitangent.2))}
	}
}
impl Div<f32> for Bitangent {
	type Output = Bitangent;
	fn div(self, rhs: f32) -> Bitangent {
		Bitangent{ bitangent:((self.bitangent.0 / rhs),
		(self.bitangent.1 / rhs),
		(self.bitangent.2 / rhs))}
	}
}
impl Div for Tangent {
	type Output = Tangent;
	fn div(self, rhs: Tangent) -> Tangent {
		Tangent{ tangent:((self.tangent.0 / rhs.tangent.0),
		(self.tangent.1 / rhs.tangent.1),(self.tangent.2 / rhs.tangent.2))}
	}
}
impl Div<f32> for Tangent {
	type Output = Tangent;
	fn div(self, rhs: f32) -> Tangent {
		Tangent{ tangent:((self.tangent.0 / rhs),
		(self.tangent.1 / rhs),
		(self.tangent.2 / rhs))}
	}
}

pub fn normalize(input:[f32; 3])-> [f32; 3] {
	let length = ((input[0] * input[0])+(input[1] * input[1])+(input[2] * input[2])).sqrt();
	return [input[0]/length, input[1]/length, input[2]/length];
}
pub fn vector_matrix_mul(vector: [f32; 3], matrix: cgmath::Matrix3<f32>) -> [f32; 3] {
	[(vector[0]*matrix.x.x)+(vector[1]*matrix.x.y)+(vector[2]*matrix.x.z),
	(vector[0]*matrix.y.x)+(vector[1]*matrix.y.y)+(vector[2]*matrix.y.z),
	(vector[0]*matrix.z.x)+(vector[1]*matrix.z.y)+(vector[2]*matrix.z.z)]
}
#[allow(dead_code)]
pub fn cross_product(first:[f32; 3], second:[f32; 3])->[f32; 3]{
	[	(first[1]*second[2])-(first[2]*second[1]),
		(first[2]*second[0])-(first[0]*second[2]),
		(first[0]*second[1])-(first[1]*second[0])]
}