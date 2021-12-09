
pub struct Signal{
	samples: Vec<f64>, 
	timestamps: Vec<f64>,
	unit: String, 
	name: String,
	comment: String, 
	raw: bool, 
}

impl Signal {
	pub fn len(&self) -> usize {
		return self.samples.len();
	}
	pub fn new(timestamps: Vec<f64>, samples: Vec<f64>, unit: String, name: String, comment: String, raw: bool) -> Self {
		Self{
			samples, 
			timestamps, 
			unit, 
			name, 
			comment, 
			raw,
		}
	}
	pub fn cut(&self, 
	start: f64, 
	end: f64, 
	include_ends: bool, 
	) -> Self{
	let mut adjusted = self.clone();
		if adjusted.len() == 0 {
			return *adjusted;
		}

		if start == end {
			return *adjusted;
		}

		let (start_index, end_index) = if include_ends {
			let start_index = self.timestamps.iter().position(|&x| start <= x).expect("Couldn't find time within the range given");
			let end_index = self.timestamps.iter().position(|&x| x <= end).expect("Couldn't find time within the range given");
			(start_index, end_index)
		} else {
			let start_index = self.timestamps.iter().position(|&x| start < x).expect("Couldn't find time within the range given");
			let end_index = self.timestamps.iter().position(|&x| x < end).expect("Couldn't find time within the range given");
			(start_index, end_index)
		};
		

		if start_index == end_index {

		}

		adjusted.timestamps = adjusted.timestamps[start_index..end_index].to_vec();
		adjusted.samples = adjusted.samples[start_index..end_index].to_vec();



		return *adjusted;
}

	pub fn extend(&self, other: Self) -> Self {

	}
}