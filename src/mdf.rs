use crate::mdf3;

pub trait MDF {
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list(&mut self);

    fn list_channels(&self);

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> TimeChannel;
}

pub struct TimeChannel {
	pub time: Vec<f64>, 
	pub data: Vec<f64>,
}

impl TimeChannel {
	pub fn new(times: Vec<mdf3::Record>, datas: Vec<mdf3::Record>) -> Self {
		let mut t = Vec::new();
		let mut d = Vec::new();

		for time in times  {
			t.push(time.extract());
		}

		for data in datas {
			d.push(data.extract());
		}

		Self{
			time: t, 
			data: d,
		}
	}

	pub fn max_time(&self) -> f64 {
		return *self.time.last().expect("Error reading time");
	}
	
}

// pub trait IDBLOCK {
// 	fn read(stream: &[u8]) ->  (usize, bool, (dyn IDBLOCK + 'static), );
// }
// pub trait DGBLOCK {
// 	fn read(stream: &[u8]) -> Self;
// }
// pub trait CNBLOCK {}
// pub trait CGBLOCK {}
