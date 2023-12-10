use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek};
use std::path::Path;

use crate::vuids::VuidCollection;

use crate::file_edits::FileEdits;

mod file_vuids;
use file_vuids::GatherVuids;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/**
Check all source code files in a rust project for directives to check VUIDs

When found, check if all VUID checks are up-to-date and add any missing VUID check

Can be run in a build script (**only for local development**), or manually.
When run manually, one argument indicating the path of the rust project with the src directory must be provided
 */
pub fn check_vuids(check_dir: &Path, vuid_collection: &VuidCollection) -> Result<()> {
    for path in check_dir.read_dir()? {
        let path = path?;
        eprintln!("{path:?}");

        if path.file_type()?.is_dir() {
            // recurse into all directories
            check_vuids(path.path().as_path(), vuid_collection)?;
        } else {
            // open and check file
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path.path().as_path())?;

            check_file(&mut file, vuid_collection)?;
        }
    }

    Ok(())
}

fn check_file(file: &mut File, vuid_collection: &VuidCollection) -> Result<()> {
    let buffer = load_file(file)?;
    let mut parser = crate::parse::RustParser::new(&buffer);

    let file_vuids = parser.parse(GatherVuids::new())?;

    let mut file_edits = FileEdits::new(&buffer);

    // for each target found in the file, compare each target's reference VUID's "version" to the corresponding VUID's "version" in the file
    // if the reference VUIDs include new VUIDs not in the file (no corresponding in file), add the new VUIDs to the file with a compile_error!("new VUID")
    // if the reference VUIDs have a higher version, compare the associated descriptions
    // if the descriptions do not match, update the description of the file VUID and add a old_description!("...") and compile_error!("updated VUID")
    for target in file_vuids.targets() {
        let reference_vuids = vuid_collection
            .get_target(target.name())
            .ok_or(format!("Can't find VUIDs for {}", target.name()))?;

        let mut insert_offset = target.start_offset();

        for (vuid, &description) in reference_vuids
            .ordered_key_value_iter()
            .expect("vuid collection must use copy keys")
        {
            // check if target has this vuid
            match target.get_vuid(vuid) {
                Some(vuid_info) => {
                    // compare versions
                    if vuid_collection.version_tuple() > vuid_info.version() {
                        // compare description
                        if description != vuid_info.description() {
                            // update description
                            file_edits.delete(vuid_info.info_start(), vuid_info.info_end());
                            file_edits.insert(
                                updated_vuid_info(
                                    vuid_collection.version_tuple(),
                                    description,
                                    vuid_info.description(),
                                ),
                                vuid_info.info_end(),
                            );
                        }
                    }
                    // I assume the vuids in the file will be in roughly the same order as in the reference
                    // after each target vuid we find in the file, update the insert offset so we insert new ones after this
                    insert_offset = vuid_info.block_end();
                }
                None => {
                    // add new vuid
                    file_edits.insert(
                        new_vuid(vuid, vuid_collection.version_tuple(), description),
                        insert_offset,
                    );
                }
            }
        }
    }

    file_edits.make_edits(file)?;

    Ok(())
}

fn new_vuid(name: &str, (major, minor, patch): (usize, usize, usize), description: &str) -> String {
    format!(
        "\n\n'{name}: {{
            check_vuids::version!(\"{major}.{minor}.{patch}\");
            check_vuids::cur_description!(\"{description}\");
            compile_error!(\"new VUID\");
        }}"
    )
}

fn updated_vuid_info(
    (major, minor, patch): (usize, usize, usize),
    new_description: &str,
    old_description: &str,
) -> String {
    format!(
        "check_vuids::version!(\"{major}.{minor}.{patch}\");
        check_vuids::cur_description!(\"{new_description}\");
        check_vuids::old_description!(\"{old_description}\");
        compile_error!(\"updated VUID\");"
    )
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

#[allow(unused)]
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

    fn visit_block_label(
        &mut self,
        _label_start: usize,
        range: crate::parse::SubStr<'a>,
    ) -> Result<()> {
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
