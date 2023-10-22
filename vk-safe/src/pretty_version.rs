use std::fmt;

// =================VkVersion===========================
#[repr(transparent)]
#[derive(Default)]
pub struct VkVersion(u32);

impl VkVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self::new_with_variant(0, major, minor, patch)
    }
    pub const fn from_triple((major, minor, patch): (u32, u32, u32)) -> Self {
        Self::new(major, minor, patch)
    }
    pub const fn new_with_variant(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self((variant << 29) | (major << 22) | (minor << 12) | (patch))
    }
    pub const fn parts(&self) -> (u32, u32, u32) {
        let parts = self.parts_with_variant();
        (parts.1, parts.2, parts.3)
    }
    pub const fn parts_with_variant(&self) -> (u32, u32, u32, u32) {
        (
            self.0 >> 29,
            (self.0 >> 22) & 0x7F,
            (self.0 >> 12) & 0x3FF,
            self.0 & 0xFFF,
        )
    }
    pub const fn raw(&self) -> u32 {
        self.0
    }
    pub const unsafe fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

impl fmt::Debug for VkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Display>::fmt(&self, f)
    }
}

impl fmt::Display for VkVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, major, minor, patch) = self.parts_with_variant();
        if variant != 0 {
            write!(f, "{major}.{minor}.{patch} - variant: {variant}")
        } else {
            write!(f, "{major}.{minor}.{patch}")
        }
    }
}

impl From<(u32, u32, u32)> for VkVersion {
    fn from((major, minor, patch): (u32, u32, u32)) -> Self {
        Self::new(major, minor, patch)
    }
}

#[cfg(test)]
mod test {
    use super::VkVersion;

    #[test]
    fn test_no_variant() {
        let v = VkVersion::new(1, 2, 3);

        println!("{v}");
        println!("{v:?}");
    }

    #[test]
    fn test_with_variant() {
        let v = VkVersion::new_with_variant(1, 1, 2, 3);

        println!("{v}");
        println!("{v:?}");
    }
}
