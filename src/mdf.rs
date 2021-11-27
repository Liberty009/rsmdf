pub trait MDF {
    fn new(filepath: &str) -> Self;

    fn read_all(&mut self);

    fn list(&mut self);

    fn list_channels(&self);

    fn read(&self, datagroup: usize, channel_grp: usize, channel: usize) -> TimeChannel;
}

pub struct TimeChannel {}

// pub trait IDBLOCK {
// 	fn read(stream: &[u8]) ->  (usize, bool, (dyn IDBLOCK + 'static), );
// }
// pub trait DGBLOCK {
// 	fn read(stream: &[u8]) -> Self;
// }
// pub trait CNBLOCK {}
// pub trait CGBLOCK {}
