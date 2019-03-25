extern crate cgmath;

//TODO:Convert to entity
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub struct Camera {
	position: [f32; 3],
	up: (f32, f32, f32),
	lookat: (f32, f32, f32),
	pub locked: bool
}
#[allow(dead_code)]
impl Camera {
	pub fn new(eye:(f32,f32,f32), look:(f32,f32,f32)) -> Camera{
		Camera{
			position:[eye.0,eye.1,eye.2],
			lookat:look,
			up:(0.0,-1.0,0.0),
			locked: false
		}
	}
	
	pub fn get_view(self)-> cgmath::Matrix4<f32>{
		cgmath::Matrix4::look_at(
				cgmath::Point3::new(self.position[0], self.position[1], self.position[2]),
				cgmath::Point3::new(self.lookat.0, self.lookat.1, self.lookat.2),
				cgmath::Vector3::new(self.up.0, self.up.1, self.up.2))
	}
	pub fn get_position(self) -> [f32; 3]{
		(self.position.clone())
	}
	pub fn set_position(&mut self, pos:[f32; 3]){
		self.position=pos;
	}
	pub fn get_lookat(self) -> [f32; 3]{
		[self.lookat.0, self.lookat.1, self.lookat.2]
	}
	pub fn set_lookat(&mut self, pos:[f32; 3]){
		self.lookat=(pos[0],pos[1],pos[2]);
	}
	pub fn get_forward(self)->[f32; 3]{
		[self.lookat.0-self.position[0], self.lookat.1-self.position[1],self.lookat.2-self.position[2]]
	}
	pub fn get_up(self)->[f32; 3]{
		[self.up.0, self.up.1, self.up.2]
	}
	pub fn move_by(&mut self, dir:[f32; 3]){
		self.lookat = (self.lookat.0 + dir[0], self.lookat.1 + dir[1], self.lookat.2 + dir[2]);
		self.position = [self.position[0]+dir[0],self.position[1]+dir[1],self.position[2]+dir[2]];
	}
	pub fn get_cross_dir(self)->[f32; 3]{
		[(self.up.2*(self.lookat.1-self.position[1]))-(self.up.1*(self.lookat.2-self.position[2])),
		(self.up.0*(self.lookat.2-self.position[2]))-(self.up.2*(self.lookat.0-self.position[0])),
		(self.up.1*(self.lookat.0-self.position[0]))-(self.up.0*(self.lookat.1-self.position[1]))]
	}
	pub fn get_locked(self)-> bool{
		self.locked
	}
}
//TODO: merge this with a better movement system
#[derive(Copy, Clone)]
pub struct Keys{
	pub x: f32,
	pub y: f32
}
impl Keys{
	pub fn get_normalized(self)-> [f32;2]{
		let length = ((self.x*self.x)+(self.y*self.y)).sqrt();
		[self.x/length,self.y/length]
	}
}