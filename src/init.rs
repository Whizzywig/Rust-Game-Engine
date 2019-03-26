extern crate vulkano;
extern crate imagesize;

use std::sync::Arc;
use std::fs::File;
use std::io::Read;

pub fn load_texture(path:String, queue:Arc<vulkano::device::Queue>) -> Arc<vulkano::image::immutable::ImmutableImage<vulkano::format::R8G8B8A8Srgb, vulkano::memory::pool::PotentialDedicatedAllocation<vulkano::memory::pool::StdMemoryPoolAlloc>>>{
	let file = File::open(&path);
	let mut file = match file {
		Ok(file) => file,
		Err(error) => {
			panic!("There was a problem opening the file: {} with error:{:?}", &path,error)
		},
	};
	let mut buffer = Vec::new();
	file.read_to_end(&mut buffer).expect("Failed to read into buffer");
	let dim = match imagesize::size(path) {
		Ok(dim) => dim,
		Err(why) => panic!("Error getting size: {:?}", why)
	};
	let image = image::load_from_memory_with_format(&buffer, image::ImageFormat::PNG).unwrap().to_rgba();
    let image_data = image.into_raw().clone();
	let (image, _) = vulkano::image::immutable::ImmutableImage::from_iter(
			image_data.iter().cloned(),
			vulkano::image::Dimensions::Dim2d{ width: dim.width as u32, height: dim.height as u32},
			vulkano::format::R8G8B8A8Srgb,
			queue.clone()
		).unwrap();
	(image)
}