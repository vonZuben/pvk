use generator::VecMap;

use crate::must_next::MustNext;
use crate::Error;
use crate::Result;

const VUIDS: &'static str = include_str!(concat!(env!("OUT_DIR"), "/vuids.txt"));

type Target = &'static str;
type Vuid = &'static str;
type Description = &'static str;
type Version = (usize, usize, usize);

pub fn parse_version(v_str: &str) -> Result<Version> {
    let mut version_parts = v_str.split('.');
    let mut version_parts = MustNext::new(&mut version_parts);
    const ERROR: &'static str = "missing version part";
    let major = version_parts.must_next(ERROR)?.parse()?;
    let minor = version_parts.must_next(ERROR)?.parse()?;
    let patch = version_parts.must_next(ERROR)?.parse()?;
    version_parts.must_not_next("unexpected extra in API version")?;

    Ok((major, minor, patch))
}

/// represents each possible line in [VUIDS]
enum VuidLine {
    Version(Version),
    Target(Target),
    Vuid(Vuid),
    Description(Description),
}

/// like FromStr, but can borrow the source str
trait RefFromStr<'a>: Sized {
    type Err;
    fn ref_from_str(s: &'a str) -> std::result::Result<Self, Self::Err>;
}

impl RefFromStr<'static> for VuidLine {
    type Err = Error;

    fn ref_from_str(s: &'static str) -> std::result::Result<Self, Self::Err> {
        match &s[..1] {
            "A" => Ok(Self::Version(parse_version(&s[2..])?)),
            "T" => Ok(Self::Target(&s[2..])),
            "V" => Ok(Self::Vuid(&s[2..])),
            "D" => Ok(Self::Description(&s[2..])),
            _ => Err("not a valid line for VUIDS")?,
        }
    }
}

/// for implementing function like parse on str, but using [RefFromStr]
trait MyParse<'a> {
    fn my_parse<F>(&'a self) -> std::result::Result<F, <F as RefFromStr>::Err>
    where
        F: RefFromStr<'a>;
}

impl<'a> MyParse<'a> for str {
    fn my_parse<F>(&'a self) -> std::result::Result<F, <F as RefFromStr>::Err>
    where
        F: RefFromStr<'a>,
    {
        F::ref_from_str(self)
    }
}

pub struct VuidCollection {
    collection: VecMap<Target, VecMap<Vuid, Description>>,
    version: Version,
}

impl VuidCollection {
    pub fn new() -> Result<Self> {
        let mut version = None;
        let mut collection = VecMap::default();
        let mut vuid = None;

        for line in VUIDS.lines().map(MyParse::my_parse) {
            let line = line?;

            use VuidLine::*;
            match line {
                Version(v) => version = Some(v),
                Target(target) => collection.push(target, VecMap::default()),
                Vuid(id) => vuid = Some(id),
                Description(description) => collection
                    .last_mut()
                    .ok_or("no target for vuid")?
                    .push_copy_key(vuid.take().ok_or("no vuid for description")?, description),
            }
        }

        Ok(Self {
            collection,
            version: version.ok_or("no version found")?,
        })
    }
    pub fn get_target<'a>(&'a self, target: &'a str) -> Option<&'a VecMap<Vuid, Description>> {
        self.collection.get(target)
    }
    pub fn version_tuple(&self) -> Version {
        self.version
    }
}
