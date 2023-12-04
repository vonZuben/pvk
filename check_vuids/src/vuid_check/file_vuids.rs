use generator::VecMap;

#[derive(Clone, Copy)]
enum CheckVisitorState {
    LookingForCheckBlock,
    GetTarget,
    LookingForVuidBlock,
    VuidBlockStart,
    LookingForVuidVersion,
    GetVersion,
    LookingForVuidDescription,
    GetDescription,
    VuidInfoEnd,
    VuidBlockEnd,
}

pub struct TargetInfo<'a> {
    target: &'a str,
    vuids: VecMap<&'a str, VuidInfo<'a>>,
    block_start: usize,
    block_end: Option<usize>,
}

impl<'a> TargetInfo<'a> {
    fn new(target: &'a str, block_start: usize) -> Self {
        Self {
            target,
            vuids: Default::default(),
            block_start,
            block_end: None,
        }
    }

    fn last_vuid_mut(&mut self, expect: &'static str) -> &mut VuidInfo<'a> {
        self.vuids.last_mut().expect(expect)
    }

    pub fn name(&self) -> &'a str {
        self.target
    }
}

#[derive(Default)]
pub struct VuidInfo<'a> {
    version: Option<(usize, usize, usize)>,
    description: Option<&'a str>,
    info_end: Option<usize>,
    block_end: Option<usize>,
}

pub struct CheckVisitor<'a> {
    state: CheckVisitorState,
    targets: Vec<TargetInfo<'a>>,
    block_depth: usize,
    check_block_depth: Option<usize>,
    vuid_block_depth: Option<usize>,
}

impl<'a> CheckVisitor<'a> {
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

impl<'a> crate::parse::RustFileVisitor<'a> for CheckVisitor<'a> {
    fn visit_string(&mut self, range: crate::parse::SubStr<'a>) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            GetVersion => {
                let vuid = self.expect_last_vuid_mut("GetVersion state: no vuid");
                assert!(vuid.version.is_none());

                let version = crate::vuids::parse_version(&range)?;
                vuid.version = Some(version);

                self.state = LookingForVuidDescription;
            }
            GetDescription => {
                let vuid = self.expect_last_vuid_mut("GetDescription state: no vuid");
                assert!(vuid.description.is_none());

                vuid.description = Some(range.inner());

                self.state = VuidInfoEnd;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_identifier(&mut self, range: crate::parse::SubStr<'a>) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            GetTarget => {
                self.targets
                    .push(TargetInfo::new(range.inner(), self.block_depth)); // TODO this should be location of block, not block depth
                self.state = LookingForVuidBlock;
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
                    self.state = GetTarget;
                    if self.block_depth < 1 {
                        Err("check_vuid!() is not in a block")?;
                    }
                    self.check_block_depth = Some(self.block_depth);
                }
            }
            LookingForVuidVersion => {
                if &*range == "version" {
                    self.state = GetVersion;
                }
            }
            LookingForVuidDescription => {
                if &*range == "description" {
                    self.state = GetDescription;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_block_label(&mut self, range: crate::parse::SubStr<'a>) -> crate::Result<()> {
        use CheckVisitorState::*;
        match self.state {
            LookingForVuidBlock => {
                self.expect_last_target_mut("LookingForVuidBlock state: no target")
                    .vuids
                    .push(range.inner(), Default::default());
                self.state = VuidBlockStart;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_delim_start(
        &mut self,
        _offset: usize,
        _kind: crate::parse::Delimiter,
    ) -> crate::Result<()> {
        use CheckVisitorState::*;
        self.block_depth += 1;
        match self.state {
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
            VuidBlockEnd => {
                let depth = self
                    .vuid_block_depth
                    .expect("VuidBlockEnd state: no vuid block");
                if depth == self.block_depth {
                    self.vuid_block_depth = None;
                    self.state = LookingForVuidBlock;

                    let vuid = self.expect_last_vuid_mut("VuidInfoEnd state: no vuid");
                    assert!(vuid.block_end.is_none());

                    vuid.block_end = Some(offset);
                }
            }
            LookingForVuidBlock => {
                let depth = self
                    .check_block_depth
                    .expect("LookingForVuidBlock state: no check block");
                if depth == self.block_depth {
                    self.vuid_block_depth = None;
                    self.state = LookingForCheckBlock;

                    let target = self.expect_last_target_mut("no target for check block");
                    assert!(target.block_end.is_none());

                    target.block_end = Some(offset);
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
            VuidInfoEnd => {
                let vuid = self.expect_last_vuid_mut("VuidInfoEnd state: no vuid");
                assert!(vuid.info_end.is_none());

                vuid.info_end = Some(offset);

                self.state = VuidBlockEnd;
            }
            _ => {}
        }
        Ok(())
    }
}
