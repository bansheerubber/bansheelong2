use serde::{Deserialize, Serialize};
use std::{
	io::{BufReader, BufWriter},
	ops::Deref,
	sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub struct Database<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	data: RwLock<T>,
	path: String,
}

impl<T> Database<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	pub fn new(path: &str) -> Self {
		Self {
			data: RwLock::new(T::default()),
			path: path.into(),
		}
	}

	pub fn get(&self) -> RwLockReadGuard<T> {
		self.data.read().unwrap()
	}

	pub fn get_mut(&self) -> RwLockWriteGuard<T> {
		self.data.write().unwrap()
	}

	pub fn save(&self) {
		let data = self.data.read().unwrap();
		let file = BufWriter::new(std::fs::File::create(&self.path).unwrap());
		serde_json::to_writer(file, data.deref()).unwrap();
	}

	pub fn load(&mut self) {
		let file = BufReader::new(std::fs::File::open(&self.path).unwrap());
		self.data = RwLock::new(serde_json::from_reader(file).unwrap());
	}
}
