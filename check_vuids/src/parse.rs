type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delimiter {
    Brace,
    Bracket,
    Parenthesis,
}

impl From<u8> for Delimiter {
    fn from(value: u8) -> Self {
        match value {
            b'{' | b'}' => Self::Brace,
            b'[' | b']' => Self::Bracket,
            b'(' | b')' => Self::Parenthesis,
            _ => panic!("ERROR: tried to convert non-delimiter character to Delimiter"),
        }
    }
}

impl std::fmt::Display for Delimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Delimiter::*;
        match self {
            Brace => write!(f, "Brace"),
            Bracket => write!(f, "Bracket"),
            Parenthesis => write!(f, "Parenthesis"),
        }
    }
}

// is_delimiter
// '{' | b'}' | b'[' | b']' | b'(' | b')' => true,

fn is_delimiter_start(byte: u8) -> bool {
    match byte {
        b'{' | b'[' | b'(' => true,
        _ => false,
    }
}

fn is_delimiter_end(byte: u8) -> bool {
    match byte {
        b'}' | b']' | b')' => true,
        _ => false,
    }
}

pub struct SubStr<'a> {
    /// sub slice of a larger slice
    sub_slice: &'a str,
    /// start position of the sub_slice within the source slice
    start: usize,
}

impl<'a> SubStr<'a> {
    fn new(sub_slice: &'a str, start: usize) -> Self {
        Self { sub_slice, start }
    }

    pub fn start_position(&self) -> usize {
        self.start
    }

    pub fn end_position(&self) -> usize {
        self.start + self.sub_slice.len()
    }
}

impl std::ops::Deref for SubStr<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.sub_slice
    }
}

/// A rust file visitor
///
/// allows different visitors to customize behavior when visiting different parts of a rust file
///
/// all visit methods should report Ok(()) to indicate that internal parsing was successful
///
/// by default, all visit methods do nothing and return Ok(()), so that different visitors only need
/// to implement visiting for the parts they care about
pub trait RustFileVisitor<'a> {
    #[allow(unused_variables)]
    fn visit_string(&mut self, range: SubStr<'a>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn visit_identifier(&mut self, range: SubStr<'a>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn visit_macro_call_identifier(&mut self, range: SubStr<'a>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn visit_block_label(&mut self, range: SubStr<'a>) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn visit_delim_start(&mut self, offset: usize, kind: Delimiter) -> Result<()> {
        Ok(())
    }
    #[allow(unused_variables)]
    fn visit_delim_end(&mut self, offset: usize, kind: Delimiter) -> Result<()> {
        Ok(())
    }
}

struct BytePositionIterator<'a> {
    // start of buffer
    buffer: *const u8,

    // len of buffer
    len: usize,

    // current offset in iterating
    offset: usize,

    // line of last returned byte based on number of newlines detected
    line: usize,

    // column (in bytes) of last return byte in line
    column_b: usize,

    // toggle to check if line should be incremented on a subsequent call to next
    next_line: bool,

    _buffer: std::marker::PhantomData<&'a str>,
}

impl<'a> BytePositionIterator<'a> {
    fn new(parser: &'a RustParser) -> Self {
        Self {
            buffer: parser.buffer.as_ptr(),
            len: parser.buffer.len(),
            offset: 0,
            line: 1,
            column_b: 0,
            next_line: false,
            _buffer: Default::default(),
        }
    }

    fn error_at(&self, e: Box<dyn std::error::Error>) -> PositionError {
        let line_start_offset = self.offset - self.column_b;

        let line_start_ptr = unsafe { self.buffer.add(line_start_offset) };

        let mut tmp_offset = self.offset;
        while unsafe { self.buffer.add(tmp_offset).read() } != b'\n' && tmp_offset < self.len {
            tmp_offset += 1;
        }

        let line_len = tmp_offset - line_start_offset;

        // safe since this should just be a substring of an already confirmed utf8 string between newline characters
        let line_text = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(line_start_ptr, line_len))
        };

        PositionError {
            file_context: line_text.to_owned(),
            column: self.column_b,
            cause: e,
        }
    }
}

struct Byte {
    value: u8,
    offset: usize,
}

impl Iterator for BytePositionIterator<'_> {
    type Item = Byte;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.len {
            return None;
        }

        let value = unsafe { self.buffer.add(self.offset).read() };

        let this_byte = Byte {
            value,
            offset: self.offset,
        };

        // set offset for returning next byte
        self.offset += 1;

        // set column of this byte
        self.column_b += 1;

        // rest line and column when last byte was new line
        if self.next_line {
            self.line += 1;
            self.column_b = 1; // since the this byte should be in the first column of the new line
            self.next_line = false;
        }

        // if this this byte is a new line, prepare to rest line for next byte
        if value == b'\n' {
            self.next_line = true;
        }

        Some(this_byte)
    }
}

struct PositionError {
    file_context: String,
    column: usize,
    cause: Box<dyn std::error::Error>,
}

impl std::fmt::Display for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "{}", self.file_context)?;
        for _ in 1..self.column {
            write!(f, " ")?;
        }
        writeln!(f, "^")?;
        writeln!(f, "{}", self.cause)
    }
}

impl std::fmt::Debug for PositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}

impl std::error::Error for PositionError {}

pub struct RustParser<'a> {
    buffer: &'a str,
}

impl<'a> RustParser<'a> {
    pub fn new(buffer: &'a str) -> Self {
        Self { buffer }
    }

    /// parse rust file as bytes
    ///
    /// since [RustParser] can be made from any buffer, we just assume it is a rust file
    /// if it is not a rust file, then it will "likely" just return Err
    ///
    /// A file with proper rust code contains raw characters in different contexts. It is
    /// important to distinguish between normal code and "strings" since a strong can contain
    /// any text
    pub fn parse<V: RustFileVisitor<'a>>(&mut self, visitor: &mut V) -> Result<()> {
        // parsing context
        let mut block_stack: Vec<Delimiter> = Vec::new();

        // since we enumerate from the beginning of the buffer, the enumerator index represents an offset into the buffer
        let mut byte_iter = BytePositionIterator::new(self);

        let mut inner_res = || -> Result<()> {
            let mut byte_iter = byte_iter.by_ref().peekable();
            while let Some(byte) = byte_iter.next() {
                match byte.value {
                    // visit a comment
                    b'/' => eat_comment(&mut byte_iter)?,
                    // visiting a string
                    b'"' => {
                        let string_start = byte.offset + 1;
                        'find_string_end: while let Some(byte) = byte_iter.next() {
                            match byte.value {
                                b'\\' => {
                                    // literal '\'
                                    // just skip next byte, since we are not parsing escape sequences
                                    // especially we do not want to detect the wrong '"' as the end
                                    byte_iter.next();
                                }
                                b'"' => {
                                    let string_end = byte.offset;
                                    visitor.visit_string(SubStr::new(
                                        &self.buffer[string_start..string_end],
                                        string_start,
                                    ))?;
                                    break 'find_string_end;
                                }
                                _ => {} // nothing, keep checking for end of string
                            }
                        }
                    }
                    // visit a marker label e.g. 'label:
                    // could be a lifetime, but it is ignored for now e.g. 'a
                    // also maybe a single char e.g. 'c'
                    b'\'' => {
                        // first check for char e.g. 'c'
                        let mut tmp_iter = self.buffer[byte.offset + 1..].bytes().peekable();
                        let skip = if tmp_iter.peek() == Some(&b'\\') {
                            2
                        } else {
                            1
                        };
                        if tmp_iter.nth(skip) == Some(b'\'') {
                            byte_iter.nth(skip);
                            continue;
                        }

                        let label_start = byte.offset + 1;
                        'find_label_end: loop {
                            match byte_iter.peek() {
                                Some(b) if is_identifier_char(b.value) => {
                                    byte_iter.next();
                                }
                                Some(b) if b.value == b':' => {
                                    let label_end = b.offset;
                                    byte_iter.next(); // consume ':'
                                    visitor.visit_block_label(SubStr {
                                        sub_slice: &self.buffer[label_start..label_end],
                                        start: label_start,
                                    })?;
                                    break 'find_label_end;
                                }
                                Some(_) => {
                                    // assumed to be a lifetime identifier
                                    // not handled at this time
                                    break 'find_label_end;
                                }
                                None => Err("ERROR: unexpected end of label")?,
                            }
                        }
                    }
                    b if is_delimiter_start(b) => {
                        let delimiter: Delimiter = b.into();
                        block_stack.push(delimiter);
                        visitor.visit_delim_start(byte.offset, delimiter)?;
                    }
                    b if is_delimiter_end(b) => {
                        let delimiter: Delimiter = b.into();
                        let previous_delimiter = block_stack
                            .pop()
                            .ok_or("ERROR: end delimiter without start")?;
                        if previous_delimiter != delimiter {
                            Err(format!(
                                "ERROR: wrong delimiter, expected '{previous_delimiter}'"
                            ))?
                        }
                        visitor.visit_delim_end(byte.offset, b.into())?;
                    }
                    b if is_identifier_start(b) => {
                        let identifier_start = byte.offset;
                        'find_identifier_end: loop {
                            match byte_iter.peek() {
                                // keep checking for more identifier chars
                                Some(identifier_char)
                                    if is_identifier_char(identifier_char.value) =>
                                {
                                    byte_iter.next();
                                }
                                // this is a macro call
                                Some(not_identifier_char) if not_identifier_char.value == b'!' => {
                                    let identifier_end = not_identifier_char.offset;
                                    visitor.visit_macro_call_identifier(SubStr {
                                        sub_slice: &self.buffer[identifier_start..identifier_end],
                                        start: identifier_start,
                                    })?;
                                    break 'find_identifier_end;
                                }
                                // end of normal identifier
                                Some(not_identifier_char) => {
                                    let identifier_end = not_identifier_char.offset;
                                    visitor.visit_identifier(SubStr {
                                        sub_slice: &self.buffer[identifier_start..identifier_end],
                                        start: identifier_start,
                                    })?;
                                    break 'find_identifier_end;
                                }
                                None => Err("ERROR: unexpected end of file")?,
                            }
                        }
                    }
                    _ => {
                        // todo!()
                    }
                }
            }
            if block_stack.len() > 0 {
                Err("ERROR: blocks missing end")?
            } else {
                Ok(())
            }
        };

        match inner_res() {
            Ok(_) => Ok(()),
            Err(e) => Err(byte_iter.error_at(e))?,
        }
    }
}

/// call this function after finding a first '/' which indicates the beginning of a comment
/// this function then determines how to eat the rest of the comment
fn eat_comment(iter: &mut impl Iterator<Item = Byte>) -> Result<()> {
    let mut iter = MustNext { iter };
    match iter.must_next("ERROR: improper comment start")?.value {
        b'/' => {
            // line comment, find end at new line
            while let Some(next_byte) = iter.next() {
                if next_byte.value == b'\n' {
                    return Ok(());
                }
            }
            Ok(()) // this means we hit the end of the file
        }
        b'*' => {
            // block comment, find end at "*/"
            const ERROR: &'static str = "ERROR: improper block comment end";
            let mut next_byte = iter.must_next(ERROR)?;
            'star_check: loop {
                if next_byte.value == b'*' {
                    next_byte = iter.must_next(ERROR)?;
                    if next_byte.value == b'/' {
                        return Ok(());
                    } else {
                        continue 'star_check;
                    }
                }
                next_byte = iter.must_next(ERROR)?;
            }
        }
        _ => Err("ERROR: improper comment syntax")?,
    }
}

// TODO this is not correct, but good enough for my own use
fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

// TODO this is not correct, but good enough for my own use
fn is_identifier_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

struct MustNext<'a, I> {
    iter: &'a mut I,
}

impl<I: Iterator> MustNext<'_, I> {
    fn must_next(&mut self, error: &'static str) -> Result<I::Item> {
        let item = self.iter.next().ok_or(error)?;
        Ok(item)
    }
}

impl<I: Iterator> Iterator for MustNext<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
