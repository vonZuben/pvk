
#[derive(Debug)]
pub struct Handle<H> {
    handle: H,
}

impl<H> Handle<H> {
    pub(crate) fn new(handle: H) -> Self {
        Self { handle }
    }
    pub fn scope<'a, R>(&'a self, f: impl FnOnce(&'a H) -> R) -> R {
        f(&self.handle)
    }
}