use crate::Result;

/// iterator must return a next element. Else Err
pub struct MustNext<'a, I> {
    iter: &'a mut I,
}

impl<'i, I: Iterator> MustNext<'i, I> {
    pub fn new(iter: &'i mut I) -> Self {
        Self { iter }
    }
    pub fn must_next(&mut self, error: &'static str) -> Result<I::Item> {
        let item = self.iter.next().ok_or(error)?;
        Ok(item)
    }
    pub fn must_not_next(&mut self, error: &'static str) -> Result<()> {
        match self.next() {
            Some(_) => Err(error)?,
            None => Ok(()),
        }
    }
}

impl<I: Iterator> Iterator for MustNext<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
