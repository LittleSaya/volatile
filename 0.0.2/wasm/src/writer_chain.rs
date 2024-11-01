use std::io::Write;

use crc32fast::Hasher;
use flate2::{write::DeflateEncoder, Compression};

pub trait GetLastBuffer {
    fn last_buffer(&self) -> &[u8];

    fn clear_buffer(&mut self);
}

pub struct CrcWriter<W: Write + GetLastBuffer> {
    hasher: Hasher,
    inner: W,
}

/// # CrcWriter
///
/// This is the #1 writer.
///
/// ```txt
/// You are here
/// |
/// CrcWriter -> CompressWriter -> TransformWriter
/// ```
/// 
/// TODO: where is the magic number 0xdebb20e3?
impl<W: Write + GetLastBuffer> CrcWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            hasher: Hasher::new(),
            inner,
        }
    }
}

impl<W: Write + GetLastBuffer> Write for CrcWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let write_result = self.inner.write(buf);
        if let Ok(bytes_written) = write_result {
            self.hasher.update(&buf[0..bytes_written]);
        }
        write_result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write + GetLastBuffer> GetLastBuffer for CrcWriter<W> {
    fn last_buffer(&self) -> &[u8] {
        self.inner.last_buffer()
    }
    
    fn clear_buffer(&mut self) {
        self.inner.clear_buffer();
    }
}

/// # CompressWriter
///
/// This is the #2 writer.
///
/// ```txt
///              You are here
///              |
/// CrcWriter -> CompressWriter -> TransformWriter
/// ```
pub struct CompressWriter<W: Write + GetLastBuffer> {
    encoder: DeflateEncoder<W>,
}

impl<W: Write + GetLastBuffer> CompressWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            encoder: DeflateEncoder::new(inner, Compression::none()),
        }
    }
}

impl<W: Write + GetLastBuffer> Write for CompressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.encoder.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.encoder.flush()
    }
}

impl<W: Write + GetLastBuffer> GetLastBuffer for CompressWriter<W> {
    fn last_buffer(&self) -> &[u8] {
        self.encoder.get_ref().last_buffer()
    }
    
    fn clear_buffer(&mut self) {
        self.encoder.get_mut().clear_buffer();
    }
}

/// # TransformWriter
///
/// This is the #3 writer.
///
/// ```txt
///                                You are here
///                                |
/// CrcWriter -> CompressWriter -> TransformWriter
/// ```
pub struct TransformWriter {
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl TransformWriter {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; 1024 * 1024 * 1024],
            buffer_size: 0,
        }
    }
}

impl Write for TransformWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // expand if necessary
        let required_buffer_size = self.buffer_size + buf.len();
        if required_buffer_size > self.buffer.len() {
            let new_size = required_buffer_size * 2;
            let mut new_buffer = vec![0; new_size];
            new_buffer[0..self.buffer_size].copy_from_slice(&self.buffer[0..self.buffer_size]);
            self.buffer = new_buffer;
        }

        // copy
        let slice = &mut self.buffer[self.buffer_size..self.buffer_size + buf.len()];
        slice.copy_from_slice(buf);

        // transform
        for i in slice.iter_mut() {
            *i = *i ^ 0xFF;
        }

        // increase buffer size
        self.buffer_size += buf.len();

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl GetLastBuffer for TransformWriter {
    fn last_buffer(&self) -> &[u8] {
        &self.buffer[0..self.buffer_size]
    }
    
    fn clear_buffer(&mut self) {
        self.buffer_size = 0;
    }
}
