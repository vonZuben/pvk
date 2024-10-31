//! SPRI-V code
//!
//! Use [`SpirvCode`] to load pre-compiled spir-v code for Vulkan shaders.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// SPIR-V code
///
/// This is an in memory buffer of pre-compiled spir-v code that has been
/// loaded and ready for creating a [`ShaderModule`](crate::vk::ShaderModule).
pub struct SpirvBinary {
    buffer: Vec<u32>,
}

impl SpirvBinary {
    /// Load SPRI-V code from a File
    ///
    /// Caller must ensure that the path points to a file containing
    /// valid pre-compiled spir-v code for use in a [`ShaderModule`](crate::vk::ShaderModule).
    ///
    /// This function does basic check to see that the amount of data
    /// read form the file is a multiple of 4.
    pub unsafe fn load_from_file_path(path: impl AsRef<Path>) -> io::Result<Self> {
        // NOTE: this is my own very basic implementation of read_to_end
        // the reason for a custom implementation is because we need a u32 buffer,
        // but read_to_end only works with u8. Although we could use read_to_end
        // and then copy the result into a u32 buffer, this can avoid the
        // unnecessary copy.

        let mut file = File::options().read(true).open(path)?;

        // spir-v must be u32 aligned
        let mut buffer: Vec<u32> = vec![0; 1024 / 4];
        // number of bytes(u8) already read
        let mut read = 0;

        // keep reading until the end of the file
        loop {
            let n = file.read(&mut buffer.as_u8_mut()[read..])?;

            read += n;

            if n == 0 {
                // if the amount read was nothing, then we are done
                break;
            } else if read == buffer.u8_len() {
                // if we reached the end of the buffer, we need to allocate more
                // reserve current len as additional len, which doubles len
                buffer.reserve(buffer.len());

                // initialize all the newly allocated space to zero
                // I want to avoid creating a &[u8] with uninitialized data to avoid any risk of UB
                // should use `read_buf` and `BorrowedBuf` when stabilized
                for b in buffer.spare_capacity_mut() {
                    b.write(0);
                }
                unsafe {
                    buffer.set_len(buffer.capacity());
                }
            }
        }

        if read % 4 != 0 || read == 0 {
            Err(io::ErrorKind::InvalidData)?;
        }

        unsafe {
            buffer.set_len(read);
        }

        Ok(Self { buffer })
    }

    pub(crate) fn code_ptr(&self) -> *const u32 {
        self.buffer.as_ptr()
    }

    pub(crate) fn code_size(&self) -> usize {
        self.buffer.u8_len()
    }
}

trait U32Buffer {
    fn as_u8_mut(&mut self) -> &mut [u8];
    fn u8_len(&self) -> usize;
}

impl U32Buffer for [u32] {
    fn as_u8_mut(&mut self) -> &mut [u8] {
        unsafe { std::mem::transmute(self) }
    }

    fn u8_len(&self) -> usize {
        self.len() * 4
    }
}
