


struct AttachmentBlock{
	id :[u8; 4], 
	reserved0 :u8, 
	block_len :u8, 
	links_nr :u8, 
	next_at_addr :u8, 
	file_name_addr :u8, 
	mime_addr :u8, 
	comment_addr :u8, 
	flags: u8, 
	creator_index : u8, 
	reserved1 : u8, 
	md5_sum : [u8; 10],
	original_size : u8, 
	embedded_size : u8, 
	embedded_data : u8,
	address : u8, 
	file_name : String, 
	mime : String, 
	comment : String,
}
struct Channel{
	id : [u8; 4],  //block ID; always b'##CN'
     reserved0 : u8, //reserved bytes
     block_len : u8, //block bytes size
     links_nr : u8, //number of links
     next_ch_addr : u8, //next ATBLOCK address
     component_addr : u8, //address of first channel in case of structure channel
    //   composition, or ChannelArrayBlock in case of arrays
    //   file name
     name_addr : u8, //address of TXBLOCK that contains the channel name
     source_addr : u8, //address of channel source block
     conversion_addr : u8, //address of channel conversion block
     data_block_addr : u8, //address of signal data block for VLSD channels
     unit_addr : u8, //address of TXBLOCK that contains the channel unit
     comment_addr : u8, //address of TXBLOCK/MDBLOCK that contains the
    //   channel comment
     attachment_addr : u8, //address of N:th ATBLOCK referenced by the
    //   current channel; if no ATBLOCK is referenced there will be no such key:value
    //   pair
     default_X_dg_addr : u8, //address of DGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
     default_X_cg_addr : u8, //address of CGBLOCK where the default X axis
    //   channel for the current channel is found; this key:value pair will not
    //   exist for channels that don't have a default X axis
     default_X_ch_addr : u8, //address of default X axis
    //   channel for the current channel; this key:value pair will not
    //   exist for channels that don't have a default X axis
     channel_type : u8, //integer code for the channel type
     sync_type : u8, //integer code for the channel's sync type
     data_type : u8, //integer code for the channel's data type
     bit_offset : u8, //bit offset
     byte_offset : u8, //byte offset within the data record
     bit_count : u8, //channel bit count
     flags : u8, //CNBLOCK flags
     pos_invalidation_bit : u8, //invalidation bit position for the current
    //   channel if there are invalidation bytes in the data record
     precision : u8, //integer code for the precision
     reserved1 : u8, //reserved bytes
     min_raw_value : u8, //min raw value of all samples
     max_raw_value : u8, //max raw value of all samples
     lower_limit : u8, //min physical value of all samples
     upper_limit : u8, //max physical value of all samples
     lower_ext_limit : u8, //min physical value of all samples
     upper_ext_limit : u8, //max physical value of all samples

    // Other attributes

     address : u8, //channel address
     attachments : list,  //list of referenced attachment blocks indexes;
    //   the index reference to the attachment block index
     comment : String, // channel comment
     conversion : ChannelConversion, // channel conversion; None if the
    //   channel has no conversion
     display_name : String, // channel display name; this is extracted from the
    //   XML channel comment
     name : String,  //channel name
     source : SourceInformation, // channel source information; None if
    //   the channel has no source information
     unit : String,// channel unit
}
struct ChannelArrayBlock{}
struct ChannelGroup{}
struct ChannelConversion{}
struct DataBlock{}
struct DataZippedBlock{}
struct DataGroup{}
struct DataList{}
struct EventBlock{}
struct FileIdentificationBlock{}
struct FileHistory{}
struct HeaderBlock{}
struct HeaderList{}
struct ListData{}
struct SourceInformation{}
struct TextBlock{}