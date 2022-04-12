#[derive(Debug, Clone)]
struct Fhblock {
    #[allow(dead_code)]
    fh_fh_next: u64,
    #[allow(dead_code)]
    fh_md_comment: u64,
    #[allow(dead_code)]
    fh_time_ns: u64,
    #[allow(dead_code)]
    fh_tz_offset_min: i16,
    #[allow(dead_code)]
    fh_dst_offset_min: i16,
    #[allow(dead_code)]
    fh_time_flags: u8,
}
impl Block for Fhblock {
    fn new() -> Self {
        Self {
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
        }
    }
    fn default() -> Self {
        Self {
            fh_fh_next: 0_u64,
            fh_md_comment: 0_u64,
            fh_time_ns: 0_u64,
            fh_tz_offset_min: 0_i16,
            fh_dst_offset_min: 0_i16,
            fh_time_flags: 0_u8,
        }
    }
    fn read(stream: &[u8], position: usize, little_endian: bool) -> (usize, Self) {
        let (pos, header) = BlockHeader::read(stream, position, little_endian);

        if !utils::eq(&header.id, "##FH".as_bytes()) {
            panic!("Error FHBLOCK");
        }

        let (mut pos, mut address) = link_extract(stream, pos, little_endian, header.link_count);

        let fh_fh_next = address.remove(0);
        let fh_md_comment = address.remove(0);

        let fh_time_ns = utils::read(stream, little_endian, &mut pos);
        let fh_tz_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_dst_offset_min = utils::read(stream, little_endian, &mut pos);
        let fh_time_flags = utils::read(stream, little_endian, &mut pos);

        (
            pos,
            Self {
                fh_fh_next,
                fh_md_comment,
                fh_time_ns,
                fh_tz_offset_min,
                fh_dst_offset_min,
                fh_time_flags,
            },
        )
    }
}