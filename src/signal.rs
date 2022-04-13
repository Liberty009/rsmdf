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
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[must_use]
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
    #[must_use]
    pub fn cut(&self, start: f64, end: f64, include_ends: bool) -> Self {
        let mut adjusted = self.clone();
        if adjusted.is_empty() {
            return adjusted;
        }

        if (start - end).abs() < 0.001 {
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

        adjusted
    }

    #[must_use]
    pub fn extend(&self, other: Self) -> Self {
        let last_stamp = if !self.is_empty() {
            *self.timestamps.last().unwrap()
        } else {
            0.0
        };

        if !other.is_empty() {
            let other_first_sample = other.timestamps[0];
            let mut timestamps = if other_first_sample <= last_stamp {
                other
                    .timestamps
                    .into_iter()
                    .map(|x| x + last_stamp)
                    .collect()
            } else {
                other.timestamps
            };

            let mut new_samples = Vec::with_capacity(self.samples.len() + other.samples.len());
            new_samples.append(&mut self.samples.clone());
            #[allow(clippy::redundant_clone)]
            new_samples.append(&mut other.samples.clone());
            let mut new_timestamps = Vec::with_capacity(self.timestamps.len() + timestamps.len());
            new_timestamps.append(&mut self.timestamps.clone());
            new_timestamps.append(&mut timestamps);

            Self {
                samples: new_samples,
                timestamps: new_timestamps,
                unit: self.unit.clone(),
                name: self.name.clone(),
                comment: self.comment.clone(),
                raw: self.raw,
            }
        } else {
            self.clone()
        }
    }

    #[must_use]
    pub fn interp(&self, new_timestamps: Vec<f64>, interpolation_mode: Interpolation) -> Self {
        if self.samples.is_empty() || new_timestamps.is_empty() {
            return self.clone();
        }

        let signal = self.clone();
        match interpolation_mode {
            Interpolation::RepeatPreviousSample => {}
            Interpolation::LinearInterpolation => {}
        }

        signal
    }

    pub fn as_type() {}

    pub fn physical() {}

    pub fn validate() {}

    #[must_use]
    pub fn copy(&self) -> Self {
        self.clone()
    }

    #[must_use]
    pub fn max_time(&self) -> f64 {
        *self.timestamps.last().expect("No time value found")
    }
}

pub enum Interpolation {
    RepeatPreviousSample,
    LinearInterpolation,
}
