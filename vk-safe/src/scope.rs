//! Distinguishable object handling
//!
//! All dispatchable handles (e.g. Instance, Device, etc.) implement all commands as methods on
//! a special **scoped** version of the particular handle. This is used to ensure different
//! handles are distinct. A particular scope is identified by its [`Tag`].
//!
//! This is because the Vulkan API exposes a complex parent child hierarchy, and
//! it must be ensured that "child" are resources only used with their respective "parent" resource
//! or "sibling" resources. In this regard, it is necessary to ensure that different instances
//! of a particular handle are distinguishable from one another.
//!
//! Use the [`tag!`] macro to create tags for tagging handles (and some other objects)
//! at construction time.
//!
//! ## Example
//! ```
//! use vk_safe::vk;
//!
//! vk::tag!(tag1);
//! vk::tag!(tag2);
//!
//! // This does nto compile since the invariant
//! // lifetimes cannot unify
//! // let _ = [tag1, tag2];
//! ```

use std::marker::PhantomData;

/// An scope anchor
///
/// Ensures a scope is bound to the location of the anchor
/// so long as you only use [`scope!`] to create it.
#[doc(hidden)]
pub struct Anchor<'anchor>(PhantomData<*mut &'anchor ()>);

impl Anchor<'_> {
    /// DO NOT USE
    ///
    /// use [`scope!`]
    ///
    /// This is just a convenience for making an Anchor in [`scope!`]
    pub unsafe fn new() -> Self {
        Self(PhantomData)
    }
}

/// Bounds of a scope
///
/// The bounds of a scope are defined from the point of creation
/// until the point of dropping.
///
/// ScopeBounds has a no-op Drop impl to ensure that it needs
/// to live until the end of the defining scope, so long
/// as you only use [`scope!`] to create it.
#[doc(hidden)]
pub struct ScopeBounds<'anchor>(PhantomData<&'anchor Anchor<'anchor>>);

impl<'anchor> ScopeBounds<'anchor> {
    /// DO NOT USE
    ///
    /// use [`scope!`]
    ///
    /// FYI, this "ties" the invariant lifetime of the Anchor to the lifetime of the
    /// ScopeBounds.
    pub unsafe fn new(_anchor: &'anchor Anchor<'anchor>) -> Self {
        Self(PhantomData)
    }
}

impl Drop for ScopeBounds<'_> {
    fn drop(&mut self) {
        // do nothing
        // The Drop impl for ScopeBounds simply ensures that
        // ScopeBounds lives from the point of creation until
        // the end of the defining scope
    }
}

/// This represents an ID for a scope based on an invariant lifetime
///
/// by creating this and passing it to a closure with Higher-Rank Trait Bound lifetime (i.e. for<'scope>)
/// Id<'scope> will uniquely mark the specific scope of the closure body
///
/// this can be used to ensure different things should have the same Id
#[derive(Default, Clone, Copy)]
#[repr(transparent)]
#[doc(hidden)]
pub struct ScopeId<'id>(PhantomData<*mut &'id ()>);

unsafe impl Send for ScopeId<'_> {}
unsafe impl Sync for ScopeId<'_> {}

/// A tag that uniquely identifies a specific region of code
///
/// Create using the [`tag!()`] macro.
///
/// Can be used in certain vk_safe apis that require maintaining
/// specific relationships between specific instances of things
pub struct Tag<'id>(ScopeId<'id>);

impl<'id> Tag<'id> {
    /// helper method
    ///
    /// do NOT use directly. Use [`tag!()`]
    #[doc(hidden)]
    pub unsafe fn new(_bounds: &ScopeBounds<'id>) -> Self {
        Self(Default::default())
    }
}

pub trait Captures<T> {}
impl<T, U> Captures<T> for U {}

/// create a scope tag
///
/// Creates a [`Tag<'id>`](Tag) with the provided name.
///
/// The created [`Tag`] will be bound the the unique region of code
/// that it is defined in (i.e. a scope). Different invocations of [`tag!`] will
/// create different unique tags.
#[macro_export]
macro_rules! tag {
    ( $name:ident ) => {
        let $name = unsafe { $crate::scope::Anchor::new() };
        let $name = unsafe { $crate::scope::ScopeBounds::new(&$name) };
        let $name = unsafe { $crate::scope::Tag::new(&$name) };
    };
}
#[doc(inline)]
pub use tag;

/// Check if a combined tag type has a particular tag
///
/// Sometimes, a type has multiple relevant tags. In order
/// to reduce the number of type parameters, the tags
/// are combined into a single type. This trait allows
/// checking individual tags from the combined tag type.
pub trait HasScope<S> {}

impl<S, U> HasScope<S> for (S, U) {}
