use generator::VecMap;

#[derive(Clone, Copy)]
enum CheckVisitorState {
    LookingForCheckBlock,
    TargetStart,
    GetTarget,
    TargetEnd,
    LookingForVuidBlock,
    VuidBlockStart,
    LookingForVuidVersion,
    VersionStart,
    GetVersion,
    VersionEnd,
    LookingForVuidDescription,
    DescriptionStart,
    GetDescription,
    DescriptionEnd,
    VuidInfoEnd,
    VuidBlockEnd,
}

pub struct TargetInfo<'a> {
    target: &'a str,
    vuids: VecMap<&'a str, VuidInfo<'a>>,

    /// offset into the file right after the check_vuids!(target) call
    vuids_start: Option<usize>,

    /// offset into the file at the end of the block which contains the vuids to check
    block_end: Option<usize>,
}

impl<'a> TargetInfo<'a> {
    fn new(target: &'a str) -> Self {
        Self {
            target,
            vuids: Default::default(),
            vuids_start: None,
            block_end: None,
        }
    }

    fn last_vuid_mut(&mut self, expect: &'static str) -> &mut VuidInfo<'a> {
        self.vuids.last_mut().expect(expect)
    }

    pub fn name(&self) -> &'a str {
        self.target
    }

    pub fn get_vuid(&self, vuid: &'a str) -> Option<&VuidInfo<'a>> {
        self.vuids.get(vuid)
    }

    pub fn start_offset(&self) -> usize {
        self.vuids_start
            .expect("Target parsed with improper syntax")
    }
}

pub struct VuidInfo<'a> {
    version: Option<(usize, usize, usize)>,
    description: Option<&'a str>,

    #[allow(unused)]
    /// offset into the file to the beginning of the vuid block label (including ')
    start: usize,

    /// offset into the file to the first byte of version!("ver");
    info_start: Option<usize>,

    /// offset into the file to the first byte after the cur_description!("desc");
    info_end: Option<usize>,

    /// offset into the file to the end of the block containing the vuid info and check code
    block_end: Option<usize>,
}

// these methods are intended to be used after the VuidInfo is already fully parsed
// thus, we assume that all the options are Some
impl<'a> VuidInfo<'a> {
    fn new(start: usize) -> Self {
        Self {
            version: None,
            description: None,
            start,
            info_start: None,
            info_end: None,
            block_end: None,
        }
    }
    pub fn version(&self) -> (usize, usize, usize) {
        self.version.expect("version must have been parsed")
    }
    pub fn description(&self) -> &'a str {
        self.description.expect("description must have been parsed")
    }
    pub fn info_start(&self) -> usize {
        self.info_start.expect("info start must have been found")
    }
    pub fn info_end(&self) -> usize {
        self.info_end.expect("info end must have been found")
    }
    pub fn block_end(&self) -> usize {
        self.block_end.expect("block end must have been found")
    }
}

pub struct GatherVuids<'a> {
    state: CheckVisitorState,
    targets: Vec<TargetInfo<'a>>,
    block_depth: usize,
    check_block_depth: Option<usize>,
    vuid_block_depth: Option<usize>,
}

impl<'a> GatherVuids<'a> {
    pub fn new() -> Self {
        Self {
            state: CheckVisitorState::LookingForCheckBlock,
            targets: Vec::new(),
            block_depth: 0,
            check_block_depth: None,
            vuid_block_depth: None,
        }
    }

    fn expect_last_target_mut(&mut self, expect: &'static str) -> &mut TargetInfo<'a> {
        self.targets.last_mut().expect(expect)
    }

    fn expect_last_vuid_mut(&mut self, expect: &'static str) -> &mut VuidInfo<'a> {
        self.expect_last_target_mut(expect).last_vuid_mut(expect)
    }

    pub fn targets(&self) -> impl Iterator<Item = &TargetInfo<'a>> {
        self.targets.iter()
    }
}

impl<'a> crate::parse::RustFileVisitor<'a> for GatherVuids<'a> {
    fn visit_string(&mut self, range: crate::parse::SubStr<'a>) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            GetVersion => {
                let vuid = self.expect_last_vuid_mut("GetVersion state: no vuid");
                assert!(vuid.version.is_none());

                let version = crate::vuids::parse_version(&range)?;
                vuid.version = Some(version);

                self.state = VersionEnd;
            }
            GetDescription => {
                let vuid = self.expect_last_vuid_mut("GetDescription state: no vuid");
                assert!(vuid.description.is_none());

                vuid.description = Some(range.inner());

                self.state = DescriptionEnd;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_identifier(&mut self, range: crate::parse::SubStr<'a>) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            GetTarget => {
                self.targets.push(TargetInfo::new(range.inner()));
                self.state = TargetEnd;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_macro_call_identifier(
        &mut self,
        range: crate::parse::SubStr<'a>,
    ) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            LookingForCheckBlock => {
                if &*range == "check_vuids" {
                    if self.block_depth < 1 {
                        Err("check_vuid!() is not in a block")?;
                    }
                    self.state = TargetStart;
                    self.check_block_depth = Some(self.block_depth);
                }
            }
            LookingForVuidVersion => {
                if &*range == "version" {
                    self.state = VersionStart;
                    self.expect_last_vuid_mut("LookingForVuidVersion no vuid")
                        .info_start = Some(range.start_position());
                } else if range.contains("description") {
                    Err("version should come before description")?
                }
            }
            LookingForVuidDescription => {
                if &*range == "cur_description" {
                    self.state = DescriptionStart;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_block_label(
        &mut self,
        label_start: usize,
        range: crate::parse::SubStr<'a>,
    ) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            LookingForVuidBlock => {
                self.expect_last_target_mut("LookingForVuidBlock state: no target")
                    .vuids
                    .push(range.inner(), VuidInfo::new(label_start));
                self.state = VuidBlockStart;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_delim_start(
        &mut self,
        _offset: usize,
        kind: crate::parse::Delimiter,
    ) -> crate::Result<()> {
        use CheckVisitorState::*;
        self.block_depth += 1;
        match self.state {
            TargetStart => {
                if kind != crate::parse::Delimiter::Parenthesis {
                    Err("Expect Parenthesis")?;
                }
                self.state = GetTarget;
            }
            VersionStart => {
                if kind != crate::parse::Delimiter::Parenthesis {
                    Err("Expect Parenthesis")?;
                }
                self.state = GetVersion;
            }
            DescriptionStart => {
                if kind != crate::parse::Delimiter::Parenthesis {
                    Err("Expect Parenthesis")?;
                }
                self.state = GetDescription;
            }
            VuidBlockStart => {
                self.state = LookingForVuidVersion;
                self.vuid_block_depth = Some(self.block_depth);
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_delim_end(
        &mut self,
        offset: usize,
        _kind: crate::parse::Delimiter,
    ) -> crate::Result<()> {
        use CheckVisitorState::*;

        match self.state {
            GetTarget => Err("check_vuids!(...) missing target")?,
            LookingForVuidVersion | GetVersion => Err("could not find version!(version_text)")?,
            LookingForVuidDescription | GetDescription => {
                Err("could not find cur_description!(description_text)")?
            }
            VersionEnd => self.state = LookingForVuidDescription,
            DescriptionEnd => self.state = VuidInfoEnd,
            VuidBlockEnd => {
                // expected that this must be found, or else the rust source file is already malformed
                // the call to end() will ensure that the proper number of block ends were found
                let depth = self
                    .vuid_block_depth
                    .expect("VuidBlockEnd state: no vuid block");
                if depth == self.block_depth {
                    self.vuid_block_depth = None;
                    self.state = LookingForVuidBlock;

                    let vuid = self.expect_last_vuid_mut("VuidInfoEnd state: no vuid");
                    assert!(vuid.block_end.is_none());

                    vuid.block_end = Some(offset + 1);
                }
            }
            LookingForVuidBlock => {
                // expected that this must be found, or else the rust source file is already malformed
                // the call to end() will ensure that the proper number of block ends were found
                let depth = self
                    .check_block_depth
                    .expect("LookingForVuidBlock state: no check block");
                if depth == self.block_depth {
                    self.check_block_depth = None;
                    self.state = LookingForCheckBlock;

                    let target = self.expect_last_target_mut("no target for check block");
                    assert!(target.block_end.is_none());

                    target.block_end = Some(offset + 1);
                }
            }
            _ => {}
        }

        self.block_depth -= 1;
        Ok(())
    }

    fn visit_semi_colon(&mut self, offset: usize) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            TargetEnd => {
                let target = self.expect_last_target_mut("TargetEnd no target");
                target.vuids_start = Some(offset + 1);
                self.state = LookingForVuidBlock;
            }
            VuidInfoEnd => {
                let vuid = self.expect_last_vuid_mut("VuidInfoEnd state: no vuid");
                assert!(vuid.info_end.is_none());

                vuid.info_end = Some(offset + 1);

                self.state = VuidBlockEnd;
            }
            _ => {}
        }
        Ok(())
    }

    fn end(&mut self) -> crate::Result<()> {
        if self.block_depth != 0 {
            Err("incorrect block delimiters")?
        } else {
            Ok(())
        }
    }
}
