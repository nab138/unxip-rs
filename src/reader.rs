use crate::UnxipError;
use apple_xar::reader::XarReader;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use xz2::read::XzDecoder;

const PBZX_MAGIC: &[u8] = b"\x70\x62\x7a\x78";
const LZMA_MAGIC: &[u8] = b"\xfd\x37\x7a\x58\x5a\x00";

pub struct XipReader<R: Read + Seek + Sized + std::fmt::Debug> {
    r: R,
    content_length: u64,
    content_read_size: u64,
    chunk_decoder: Option<Box<dyn Read>>,
    done: bool,
    flags: u64,
}

impl<R: Read + Seek + Sized + std::fmt::Debug> XipReader<R> {
    pub fn new(mut r: R) -> Result<Self, UnxipError> {
        let xar_reader = XarReader::new(&mut r).map_err(UnxipError::XarError)?;
        let content = xar_reader
            .find_file("Content")
            .map_err(UnxipError::XarError)?
            .ok_or_else(|| UnxipError::Misc("No Content file found".to_string()))?;
        let data = content
            .data
            .ok_or_else(|| UnxipError::Misc("No data in Content file".to_string()))?;
        let start_offset = data.offset + xar_reader.heap_start_offset();
        let content_length = data.length;
        r.seek(SeekFrom::Start(start_offset))
            .map_err(UnxipError::IoError)?;

        let mut magic = [0u8; 4];
        r.read_exact(&mut magic).map_err(UnxipError::IoError)?;
        if magic != PBZX_MAGIC {
            return Err(UnxipError::Misc("Bad PBZX magic".to_string()));
        }

        let mut flags = [0u8; 8];
        r.read_exact(&mut flags).map_err(UnxipError::IoError)?;
        let flags = u64::from_be_bytes(flags);

        Ok(Self {
            r,
            content_length,
            content_read_size: 12,
            chunk_decoder: None,
            done: false,
            flags,
        })
    }

    fn load_next_chunk(&mut self) -> Result<(), UnxipError> {
        if self.content_read_size >= self.content_length || (self.flags & (1 << 24)) == 0 {
            self.done = true;
            return Ok(());
        }
        let mut chunk_header = [0u8; 16];
        self.r
            .read_exact(&mut chunk_header)
            .map_err(UnxipError::IoError)?;
        self.content_read_size += 16;
        let flags = u64::from_be_bytes(chunk_header[0..8].try_into().unwrap());
        let size = u64::from_be_bytes(chunk_header[8..16].try_into().unwrap());
        self.flags = flags;
        let mut bytes = vec![0u8; size as usize];
        self.r.read_exact(&mut bytes).map_err(UnxipError::IoError)?;
        self.content_read_size += size;

        if size != 1 << 24 {
            if &bytes[0..6] != LZMA_MAGIC {
                return Err(UnxipError::Misc("Bad LZMA magic".to_string()));
            }
            let decoder = XzDecoder::new(Cursor::new(bytes));
            self.chunk_decoder = Some(Box::new(decoder));
        } else {
            self.chunk_decoder = Some(Box::new(Cursor::new(bytes)));
        }
        Ok(())
    }
}

impl<R: Read + Seek + Sized + std::fmt::Debug> Read for XipReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0);
        }
        loop {
            if let Some(decoder) = self.chunk_decoder.as_mut() {
                let n = decoder.read(buf)?;
                if n > 0 {
                    return Ok(n);
                }
                self.chunk_decoder = None;
            }

            match self.load_next_chunk() {
                Ok(()) => {
                    if self.done {
                        return Ok(0);
                    }
                    continue;
                }
                Err(_) => {
                    self.done = true;
                    return Ok(0);
                }
            }
        }
    }
}
