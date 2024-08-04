use serde::{Deserialize, Serialize};
use std::{
	cell::{Ref, RefCell, RefMut},
	io::{BufReader, BufWriter},
};

pub struct Database<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	data: RefCell<T>,
	path: String,
}

impl<T> Database<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	pub fn new(path: &str) -> Self {
		Self {
			data: RefCell::new(T::default()),
			path: path.into(),
		}
	}

	pub fn get(&self) -> Ref<T> {
		self.data.borrow()
	}

	pub fn get_mut(&self) -> RefMut<T> {
		self.data.borrow_mut()
	}

	pub fn save(&self) {
		let file = BufWriter::new(std::fs::File::create(&self.path).unwrap());
		serde_json::to_writer(file, &self.data).unwrap();
	}

	pub fn load(&mut self) {
		let file = BufReader::new(std::fs::File::open(&self.path).unwrap());
		self.data = RefCell::new(serde_json::from_reader(file).unwrap());
	}
}
