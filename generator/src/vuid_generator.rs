use crate::utils;
use crate::vuid;
use crate::vuid_visitor::VuidVisitor;

use krs_quote::krs_quote;

#[derive(Default)]
pub struct VuidGenerator<'a> {
    // vuid
    vuids: vuid::Vuids<'a>,
}

impl VuidGenerator<'_> {
    pub fn vuids(&self) -> String {
        let vuids = &self.vuids;
        krs_quote!({@vuids}).to_string()
    }
}

impl<'a> VuidVisitor<'a> for VuidGenerator<'a> {
    fn visit_vuid(&mut self, vuid: crate::vuid_visitor::VuidPair<'a>) {
        // get target from vuid name
        let mut name_parts = vuid.name().split("-");

        // vuid name format should be "VUID-Target-parameter_of_target-info"
        // we just need target
        assert_eq!(name_parts.next(), Some("VUID"));
        let target: utils::VkTyName = name_parts
            .next()
            .expect("error: could not get vuid target")
            .into();

        self.vuids.insert_vuid(target, vuid);
    }
    fn visit_vuid_version(&mut self, version: (u32, u32, u32)) {
        self.vuids.api_version(version)
    }
}
