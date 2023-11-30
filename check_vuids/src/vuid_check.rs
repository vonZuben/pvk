use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/**
Check all source code files in a rust project for directives to check VUIDs

When found, check if all VUID checks are up-to-date and add any missing VUID check

Can be run in a build script (**only for local development**), or manually.
When run manually, one argument indicating the path of the rust project with the src directory must be provided
 */
pub fn check_vuids(check_dir: &Path) -> Result<()> {
    for path in check_dir.read_dir()? {
        let path = path?;
        eprintln!("{path:?}");

        if path.file_type()?.is_dir() {
            // recurse into all directories
            check_vuids(path.path().as_path())?;
        } else {
            // open and check file
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path.path().as_path())?;

            check_file(&mut file)?;
        }
    }

    Ok(())
}

fn check_file(file: &mut File) -> Result<()> {
    let buffer = load_file(file)?;
    let mut parser = crate::parse::RustParser::new(&buffer);

    let mut visitor = PrintlnVisitor;

    parser.parse(&mut visitor)?;

    Ok(())
}

fn load_file(file: &mut File) -> Result<String> {
    file.seek(std::io::SeekFrom::Start(0))?;

    let file_size = file.metadata()?.len();

    let mut buffer = String::new();
    let read = file.read_to_string(&mut buffer)?;

    if read != file_size.try_into()? {
        Err("ERROR: could not read whole file")?;
    }

    Ok(buffer)
}

struct BufferCursor<'a> {
    /// buffer that the cursor points into
    buffer: &'a [u8],
    /// byte offset from beginning of buffer
    pos: usize,
}

impl<'a> BufferCursor<'a> {
    fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, pos: 0 }
    }
}

/*
macro_rules! check {
    ($name:ident) => {};
}

macro_rules! description {
    ($desc:literal) => {};
}

#[allow(unused_labels)]
const fn tst() {
    check!(CreateInstance);

    'VUID_INFO_003245: {
        version!(1.3.24);
        description!("Info must be valid");

        compile_error!("check new VUID_INFO_003245")
    }
}
 */

struct PrintlnVisitor;

impl<'a> crate::parse::RustFileVisitor<'a> for PrintlnVisitor {
    fn visit_string(&mut self, range: crate::parse::SubStr<'a>) -> Result<()> {
        let s: &str = &range;
        println!("string: {}", s);
        Ok(())
    }

    fn visit_identifier(&mut self, range: crate::parse::SubStr<'a>) -> Result<()> {
        let s: &str = &range;
        println!("id: {}", s);
        Ok(())
    }

    fn visit_macro_call_identifier(&mut self, range: crate::parse::SubStr<'a>) -> Result<()> {
        let s: &str = &range;
        println!("macro call: {}", s);
        Ok(())
    }

    fn visit_block_label(&mut self, range: crate::parse::SubStr<'a>) -> Result<()> {
        let s: &str = &range;
        println!("label: {}", s);
        Ok(())
    }

    fn visit_delim_start(&mut self, offset: usize, kind: crate::parse::Delimiter) -> Result<()> {
        println!("begin {} {}", kind, offset);
        Ok(())
    }

    fn visit_delim_end(&mut self, offset: usize, kind: crate::parse::Delimiter) -> Result<()> {
        println!("end {} {}", kind, offset);
        Ok(())
    }
}
