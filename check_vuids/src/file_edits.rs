use std::convert::TryInto;
use std::fs::File;
use std::io::IoSlice;
use std::io::{Seek, SeekFrom, Write};

use crate::Result;

pub struct FileEdits<'a> {
    /// contents of the file before editing, loaded into memory
    original_file_content: &'a str,

    /// starting offset of the edits to make
    /// when writing the edits to the file, seek to this location before writing
    start_offset: Option<usize>,

    /// offset of last edit insertion
    /// when inserting new text,
    last_edit_offset: Option<usize>,

    /// list of IoSlices to write to the file
    to_write: Vec<IoSlice<'a>>,

    /// buffers for holding new text
    ///
    /// self.to_write will include pointers to the Strings stored here
    /// since that makes the struct self referential, there is need of some internal unsafe code
    ///
    /// Invariant:
    /// 1) the String must never be modified after it is inserted to the buffer, so the String internal pointer is stable
    /// 2) self.to_write should be declared before self.buffers in Self so that self.to_write is dropped before self.buffers according to Rust Drop Order
    ///     this ensures that self.to_write references are never dangling even when dropping
    buffers: Vec<String>,
}

impl<'a> FileEdits<'a> {
    pub fn new(original_file_content: &'a str) -> Self {
        Self {
            original_file_content,
            start_offset: None,
            last_edit_offset: None,
            to_write: Default::default(),
            buffers: Default::default(),
        }
    }
    /// insert new text into the file at a given offset
    /// all insertions should be after the previous edits
    /// if insert_offset is > original_file.len(), then just append the edit to the end of the file
    pub fn insert(&mut self, edit: impl Into<String>, insert_offset: usize) {
        // record where edits start to know where to start writing to the file
        if self.start_offset.is_none() {
            self.start_offset = Some(insert_offset);
        }
        // save a buffer for the original file text between edits so that it can be written back properly
        if let Some(last_edit_offset) = self.last_edit_offset {
            assert!(last_edit_offset <= insert_offset);
            if last_edit_offset < self.original_file_content.len() {
                self.to_write.push(IoSlice::new(
                    &self.original_file_content.as_bytes()[last_edit_offset..insert_offset],
                ));
            }
        }
        self.last_edit_offset = Some(insert_offset);

        self.buffers.push(edit.into());
        let internal_buffer = unsafe { self.buffers.last().unwrap_unchecked() };
        let internal_buffer_ptr = internal_buffer.as_ptr();
        let internal_buffer_len = internal_buffer.len();

        self.to_write.push(IoSlice::new(unsafe {
            std::slice::from_raw_parts(internal_buffer_ptr, internal_buffer_len)
        }));
    }
    /// delete text from the file
    /// all deletions should be after the last edit
    pub fn delete(&mut self, from: usize, to: usize) {
        assert!(from < to);
        assert!(to < self.original_file_content.len());
        // record where edits start to know where to start writing to the file
        if self.start_offset.is_none() {
            self.start_offset = Some(from);
        }
        // save a buffer for the original file text between edits so that it can be written back properly
        if let Some(last_edit_offset) = self.last_edit_offset {
            assert!(last_edit_offset <= from);
            self.to_write.push(IoSlice::new(
                &self.original_file_content.as_bytes()[last_edit_offset..from],
            ));
        }
        // by setting last_edit_offset to "to", the portion of the original file to tbe deleted will be skipped over when writing back
        self.last_edit_offset = Some(to)
    }

    /// make the edits to the file
    /// should only be called with the correct file that contains the contents of self.original_file
    /// otherwise the resulting file will not have the intended content
    pub fn make_edits(&mut self, file: &mut File) -> Result<()> {
        match (self.start_offset, self.last_edit_offset) {
            (Some(start_offset), Some(last_edit_offset)) => {
                // insert the rest of the file from the last edit the original file to write back
                self.to_write.push(IoSlice::new(
                    &self.original_file_content.as_bytes()
                        [last_edit_offset..self.original_file_content.len()],
                ));

                // seek to the start the edits
                file.seek(SeekFrom::Start(start_offset.try_into()?))?;

                file.write_vectored(&self.to_write)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
