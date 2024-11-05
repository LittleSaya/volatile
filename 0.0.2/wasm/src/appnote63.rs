use web_sys::{js_sys, wasm_bindgen};
use wasm_bindgen::prelude::*;

/// # FileHeader
///
/// This type contains necessary fields for both "local file header" and
/// "central directory header" because they have many similar fields.
#[wasm_bindgen]
pub struct FileHeader {
    version_made_by                     : u16, // CDH only
    version_needed_to_extract           : u16, // LFH & CDH
    general_purpose_bit_flag            : u16, // LFH & CDH
    compression_method                  : u16, // LFH & CDH
    last_mod_file_time                  : u16, // LFH & CDH
    last_mod_file_date                  : u16, // LFH & CDH
    crc_32                              : u32, // LFH & CDH
    compressed_size                     : u32, // LFH & CDH
    uncompressed_size                   : u32, // LFH & CDH
    file_name_length                    : u16, // LFH & CDH
    extra_field_length                  : u16, // LFH & CDH
    file_comment_length                 : u16, // CDH only
    disk_number_start                   : u16, // CDH only
    internal_file_attributes            : u16, // CDH only
    external_file_attributes            : u32, // CDH only
    relative_offset_of_local_header     : u32, // CDH only
    file_name: Vec<u8>,
    extra_field: [u8; 32],
    file_comment: [u8; 0],
}

impl FileHeader {
    pub fn new(file_name: String, lfh_pos: u64) -> Self {
        // File name could be a relative path, which MUST not contain a driver or device
        // letter, or a leading slash. All slashes MUST be '/'.
        // Ref 4.4.17
        //
        // TODO: Optimizable, because String::replace will create a new String,
        // which is unnecessary.
        let file_name = file_name.replace('\\', "/");
        let file_name = file_name.trim_start_matches('/');
        let file_name: Vec<u8> = file_name.into();

        // Currently, this program will always use "Zip64 extended information extra field"
        // , its size is 32 bytes.
        let mut extra_field: [u8; 32] = [0; 32];

        // Note: all fields stored in Intel low-byte/high-byte order.
        //
        //         Value                    Size       Description
        //         -----                    ----       -----------
        // (ZIP64) 0x0001                   2 bytes    Tag for this "extra" block type
        //         Size                     2 bytes    Size of this "extra" block
        //         Original Size            8 bytes    Original uncompressed file size
        //         Compressed Size          8 bytes    Size of compressed data
        //         Relative Header Offset   8 bytes    Offset of local header record
        //         Disk Start Number        4 bytes    Number of the disk on which this file starts
        extra_field[0..2].copy_from_slice(&0x0001_u16.to_le_bytes());
        extra_field[2..4].copy_from_slice(&28_u16.to_le_bytes());
        // I guess they are zero, because the reading of these sizes are moved
        // from "local file header" to "extra field", so the zeros should move
        // from "local file header" to "extra field" as well.
        extra_field[4..12].copy_from_slice(&0_u64.to_le_bytes());
        extra_field[12..20].copy_from_slice(&0_u64.to_le_bytes());
        extra_field[20..28].copy_from_slice(&lfh_pos.to_le_bytes());
        // Splitting is not support, so this is hard-coded to 0, currently.
        extra_field[28..32].copy_from_slice(&0_u32.to_le_bytes());

        Self {
            // Appnote version 6.3 is referenced while writing these codes.
            // Ref 4.4.2 & 4.4.3
            version_made_by                 : 63_u16,

            // The newest feature I need is "zip64", which requires a version number of 4.5
            // Ref 4.4.2 & 4.4.3
            version_needed_to_extract       : 45_u16,

            // (starting from 0)
            // Bit 1 & 2 are kept zero because they stand for "Normal compression option".
            // Bit 3 is set because this will enable the "data descriptor".
            // Bit 11 is set because this indicates the file name and comment are utf-8 encoded.
            // Ref 4.4.4
            general_purpose_bit_flag: 0b0001_0000_0001_0000_u16,

            // Method 8 is Deflate.
            // Ref 4.4.5
            compression_method: 8_u16,

            // TODO
            last_mod_file_date: 0_u16,

            // TODO
            last_mod_file_time: 0_u16,

            // The actual crc is stored in "data descriptor".
            // Ref 4.4.4
            crc_32: 0_u32,

            // LFH: Because this program always uses "zip64", these 2 fields are set to 0xFFFFFFFF
            // here, the "zero" values will be placed in extra data, which indicates that the
            // actual compressed size and uncompressed size are stored in "data descriptor".
            //
            // CDH: Both set to 0xFFFFFFFF because of "zip64".
            compressed_size: 0xFFFFFFFF_u32,
            uncompressed_size: 0xFFFFFFFF_u32,

            file_name_length: file_name.len() as u16,

            // This is fixed because the size of "Zip64 extended information extra field" is fixed.
            extra_field_length: 32_u16,

            // No file will have comment, currently.
            file_comment_length: 0_u16,

            // "zip64", in extra data.
            disk_number_start: 0xFFFF_u16,

            // TODO: I don't understand the description of this field.
            // Ref 4.4.14
            internal_file_attributes: 0_u16,

            // TODO: I don't know how to use this field.
            // Ref 4.4.15
            external_file_attributes: 0_u32,

            // "zip64", in extra data.
            relative_offset_of_local_header: 0xFFFFFFFF_u32,

            file_name,

            extra_field,

            file_comment: [0; 0],
        }
    }

    pub fn write_into_as_lfh(&self, buffer: &js_sys::ArrayBuffer) -> js_sys::Uint8Array {
        let FileHeader {
            version_needed_to_extract,
            general_purpose_bit_flag,
            compression_method,
            last_mod_file_time,
            last_mod_file_date,
            crc_32,
            compressed_size,
            uncompressed_size,
            file_name_length,
            extra_field_length,
            file_name,
            extra_field,
            ..
        } = self;

        let local_file_header_signature = 0x04034b50_u32.to_le_bytes();
        let version_needed_to_extract = version_needed_to_extract.to_le_bytes();
        let general_purpose_bit_flag = general_purpose_bit_flag.to_le_bytes();
        let compression_method = compression_method.to_le_bytes();
        let last_mod_file_time = last_mod_file_time.to_le_bytes();
        let last_mod_file_date = last_mod_file_date.to_le_bytes();
        let crc_32 = crc_32.to_le_bytes();
        let compressed_size = compressed_size.to_le_bytes();
        let uncompressed_size = uncompressed_size.to_le_bytes();
        let file_name_length = file_name_length.to_le_bytes();
        let extra_field_length = extra_field_length.to_le_bytes();

        let total_bytes =
            local_file_header_signature.len() +
            version_needed_to_extract.len() +
            general_purpose_bit_flag.len() +
            compression_method.len() +
            last_mod_file_time.len() +
            last_mod_file_date.len() +
            crc_32.len() +
            compressed_size.len() +
            uncompressed_size.len() +
            file_name_length.len() +
            extra_field_length.len() +
            file_name.len() +
            extra_field.len();

        let mut source = Vec::with_capacity(total_bytes);

        source.extend_from_slice(&local_file_header_signature);
        source.extend_from_slice(&version_needed_to_extract);
        source.extend_from_slice(&general_purpose_bit_flag);
        source.extend_from_slice(&compression_method);
        source.extend_from_slice(&last_mod_file_time);
        source.extend_from_slice(&last_mod_file_date);
        source.extend_from_slice(&crc_32);
        source.extend_from_slice(&compressed_size);
        source.extend_from_slice(&uncompressed_size);
        source.extend_from_slice(&file_name_length);
        source.extend_from_slice(&extra_field_length);
        source.extend_from_slice(file_name);
        source.extend_from_slice(extra_field);

        let target = js_sys::Uint8Array::new_with_byte_offset_and_length(&buffer, 0, source.len() as u32);

        target.copy_from(&source);

        target
    }
}
