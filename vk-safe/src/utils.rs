use std::fmt;

#[repr(transparent)]
#[derive(Default)]
pub struct VkVersion(u32);

impl VkVersion {
    pub fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self( (variant << 29) | (major << 22) | (minor << 12) | (patch) )
    }
    pub fn parts(&self) -> (u32, u32, u32, u32) {
        (self.0 >> 29, (self.0 >> 22) & 0x7F, (self.0 >> 12) & 0x3FF, self.0 & 0xFFF)
    }
    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl fmt::Debug for VkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, major, minor, patch) = self.parts();
        write!(f, "{}.{}.{}.{}", variant, major, minor, patch)
    }
}

impl From<(u32, u32, u32)> for VkVersion {
    fn from((major, minor, patch): (u32, u32, u32)) -> Self {
        Self::new(0, major, minor, patch)
    }
}

