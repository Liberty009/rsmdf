use crate::utils;

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub trigger_time: f64,
    pub pre_trigger_time: f64,
    pub post_trigger_time: f64,
}

impl Event {
    #[allow(dead_code)]
    pub fn write() {}
    pub fn read(stream: &[u8], position: usize, little_endian: bool) -> (Event, usize) {
        let mut pos = position;
        let trigger_time = utils::read(stream, little_endian, &mut pos);
        let pre_trigger_time = utils::read(stream, little_endian, &mut pos);
        let post_trigger_time = utils::read(stream, little_endian, &mut pos);
        (
            Event {
                trigger_time,
                pre_trigger_time,
                post_trigger_time,
            },
            position,
        )
    }
}

#[cfg(test)]
mod event_test {

    #[test]
    fn read() {}

    #[test]
    fn write() {}
}
