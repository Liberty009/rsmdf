#[derive(Debug, Clone)]
pub struct Signal {
    pub samples: Vec<f64>,
    pub timestamps: Vec<f64>,
    pub unit: String,
    pub name: String,
    pub comment: String,
    pub raw: bool,
}

impl Signal {
    pub fn len(&self) -> usize {
        return self.samples.len();
    }
    pub fn new(
        timestamps: Vec<f64>,
        samples: Vec<f64>,
        unit: String,
        name: String,
        comment: String,
        raw: bool,
    ) -> Self {
        Self {
            samples,
            timestamps,
            unit,
            name,
            comment,
            raw,
        }
    }
    pub fn cut(&self, start: f64, end: f64, include_ends: bool) -> Self {
        let mut adjusted = self.clone();
        if adjusted.len() == 0 {
            return adjusted;
        }

        if start == end {
            return adjusted;
        }

        let (start_index, end_index) = if include_ends {
            let start_index = self
                .timestamps
                .iter()
                .position(|&x| start <= x)
                .expect("Couldn't find time within the range given");
            let end_index = self
                .timestamps
                .iter()
                .position(|&x| x <= end)
                .expect("Couldn't find time within the range given");
            (start_index, end_index)
        } else {
            let start_index = self
                .timestamps
                .iter()
                .position(|&x| start < x)
                .expect("Couldn't find time within the range given");
            let end_index = self
                .timestamps
                .iter()
                .position(|&x| x < end)
                .expect("Couldn't find time within the range given");
            (start_index, end_index)
        };

        if start_index == end_index {}

        adjusted.timestamps = adjusted.timestamps[start_index..end_index].to_vec();
        adjusted.samples = adjusted.samples[start_index..end_index].to_vec();

        return adjusted;
    }

    pub fn extend(&self, other: Self) -> Self {
        let last_stamp = if self.len() != 0 {
            *self.timestamps.last().unwrap()
        } else {
            0.0
        };

        if other.len() != 0 {
            let other_first_sample = other.timestamps[0];
            let timestamps = if other_first_sample <= last_stamp {
                other
                    .timestamps
                    .into_iter()
                    .map(|x| x + last_stamp)
                    .collect()
            } else {
                other.timestamps
            };

            let mut new_samples = Vec::new();
            new_samples.append(&mut self.samples.clone());
            new_samples.append(&mut other.samples.clone());
            let mut new_timestamps = Vec::new();
            new_timestamps.append(&mut self.timestamps.clone());
            new_timestamps.append(&mut timestamps.clone());

            return Self {
                samples: new_samples,
                timestamps: new_timestamps,
                unit: self.unit.clone(),
                name: self.name.clone(),
                comment: self.comment.clone(),
                raw: self.raw,
            };
        } else {
            return self.clone();
        }
    }

    pub fn interp(&self, new_timestamps: Vec<f64>, interpolation_mode: Interpolation) -> Self {
        if self.samples.len() == 0 || new_timestamps.len() == 0 {
            return self.clone();
        }

        let mut signal = self.clone();
        match interpolation_mode {
            Interpolation::RepeatPreviousSample => {}
            Interpolation::LinearInterpolation => {}
        }

        return signal;
    }

    pub fn as_type() {}
    pub fn physical() {}
    pub fn validate() {}
    pub fn copy(&self) -> Self {
        return self.clone();
    }

    pub fn max_time(&self) -> f64 {
        return *self.timestamps.last().unwrap();
    }
}

pub enum Interpolation {
    RepeatPreviousSample,
    LinearInterpolation,
}
