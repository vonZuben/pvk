use std::ffi::c_char;

use crate::generated as vk;

mod private {
    pub struct CommandComparator;
}

/// const fn for comparing s strings
///
/// SAFETY : input must be valid c string pointers
const unsafe fn cmd_name_equal(mut c1_name: *const c_char, mut c2_name: *const c_char) -> bool {
    while *c1_name != '\0' as i8 && *c1_name == *c2_name {
        c1_name = c1_name.add(1);
        c2_name = c2_name.add(1);
    }
    *c1_name == *c2_name
}

unsafe impl<C1: vk::VulkanCommand, C2: vk::VulkanCommand> krs_hlist::Comparator<C1, C2> for private::CommandComparator {
    const EQUAL: bool = unsafe { cmd_name_equal(C1::VK_NAME, C2::VK_NAME) };
}

pub trait GetCommand<C> : krs_hlist::Get<C, private::CommandComparator> {}
impl<C, L> GetCommand<C> for L where L: krs_hlist::Get<C, private::CommandComparator> {}


// impl convenience functionality for VkResult
impl crate::generated::Result {
    pub fn is_err(&self) -> bool {
        self.0 < 0
    }
    pub fn is_success(&self) -> bool {
        self.0 >= 0
    }
}