/*!
All dispatchable handles (e.g. Instance, Device, etc.) implement all commands as methods on
a special **scoped** version of the particular handle. This is used to ensure different
handles are distinct.

This is because the Vulkan API exposes a complex parent child hierarchy, and
it must be ensured "child" are resources only used with their respective "parent" resource
or "sibling" resources. In this regard, it is necessary to ensure that different instances
of a particular handle are unique from one another.

Use the [`scope!`] macro to created scoped versions of handles.

## Example
```
use vk_safe::vk;

// merely illustrative
// would be a real Instance, Device, etc.
let instance1 = ();
vk::scope!(instance1);

let instance2 = ();
vk::scope!(instance2);

// below will fail to compile because an array
// requires the same types, and these are not
// due to different invariant lifetimes
// let _ = [instance1, instance2];
```
*/

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
        // ScopeBounds lives from creation point of creation until
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

/** A scoped type

*This is an implementation detail and you are not intended to directly use this*.

Types which are only safe to use within a scope implement their methods through this wrapper, or more preferably,
through the Deref Target [`RefScope`].

#### implementation details
This is just informative, and not to be relied upon, as it could change. `Scope` is `#[repr(transparent)]`, and is a
wrapper around `&'scope T`, and a `PhantomData` type to make trh lifetime invariant. It is beneficial to be a simple
reference because it is very light weight for passing around. The lifetime will be included in any event, so even if
the T is owned, there is no benefit over just being a reference.
*/
#[repr(transparent)]
pub struct Scope<'scope, T> {
    handle: &'scope T,
    _id: ScopeId<'scope>,
}

impl<T> Clone for Scope<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Scope<'_, T> {}

impl<T: std::fmt::Debug> std::fmt::Debug for Scope<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'id, T> Scope<'id, T> {
    pub unsafe fn new(handle: &'id T, _bounds: &ScopeBounds<'id>) -> Self {
        Self {
            handle,
            _id: Default::default(),
        }
    }
}

impl<T> std::ops::Deref for Scope<'_, T> {
    type Target = RefScope<Self, T>;
    fn deref(&self) -> &Self::Target {
        // this is safe because:
        // - RefScope repr(transparent) wrapper for H
        // - Scope repr(transparent) wrapper for &'_ H
        // - Scope == &RefScope == &'_ H
        unsafe { std::mem::transmute::<Self, &RefScope<Self, T>>(*self) }
    }
}

/// create a scoped type
///
/// Creates a [`Scope<'scope, T>`](Scope) with a provided `T`.
///
/// The created [`Scope`] will be bound the the unique region of code
/// that it is defined in (i.e. a scope). Different invocations of [`scope!`] will
/// create different unique scopes.
///
/// See module level docs for more details.
#[macro_export]
#[doc(hidden)]
macro_rules! scope {
    ( $name:ident ) => {
        let anchor = unsafe { $crate::scope::Anchor::new() };
        let bounds = unsafe { $crate::scope::ScopeBounds::new(&anchor) };
        let $name = unsafe { $crate::scope::Scope::new(&$name, &bounds) };
    };
}
#[doc(inline)]
pub use scope;

/** Deref target for Scope<'_, T>

*This is an implementation detail and you are not intended to directly use this*.

Types which are only safe to use within a scope implement their methods through this wrapper.

#### implementation details
This is just informative, and not to be relied upon, as it could change. `RefScope` is a
`#[repr(transparent)]` wrapper around a `T: ?Sized`, and a `PhantomData<S>`. [`Scope<'_, T>`](Scope)
implements [`Deref`](core::ops::Deref) with `Target = RefScope<Self, T>`. In this way,
`RefScope` can only ever exist as a reference bound to the lifetime of a [`Scope`], and
the invariant lifetime information is capture in the generic `S` type parameter.

vk-safe APIs can ensure different handles have the same scope (i.e. have the same Instance
or Device parent handle) by using the same generic parameter `S`.

# Safety
`RefScope` is VERY delicate. It is only ever sound to have an instance of RefScope which is
created from dereferencing `Scope`. In this way, for some handle `T` and lifetime `'scope`,
the concrete type will ALWAYS be `RefScope<Scope<'scope, T>, T>`, even though
`Scope<'scope, T>` is abstracted away as a generic parameter `S`.
 */
#[repr(transparent)]
pub struct RefScope<S, T: ?Sized> {
    scope: PhantomData<S>,
    handle: T,
}

impl<S, T> RefScope<S, T> {
    /// Get the original Scope<'_, T>
    pub(crate) fn as_scope(&self) -> S {
        // Here, we want to reverse the Deref of Scope<'_, T> such as by transmute::<&RefScope<S, T>, S> (see Deref for Scope).
        // However, the compiler cannot know that S is correctly sized for all S,
        // but WE know that S is ALWAYS `Scope<'_, T>` (or else it was improperly constructed).
        // Thus, we can use transmute_copy.
        unsafe { std::mem::transmute_copy(&self) }
    }

    /// manually get Deref target
    ///
    /// this is helpful because rust-analyzer seems to have trouble with autocompletion
    /// in some more complex uses of RefScope, when going through auto-deref
    pub(crate) fn deref(&self) -> &T {
        &self
    }
}

impl<S, H> std::ops::Deref for RefScope<S, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

//===============================
// traits to abstract over scoped objects

/// used to get the lifetime of a scoped object back for generic impls
#[doc(hidden)]
pub trait ScopeLife<'l>: Scoped {}

impl<'l, T> ScopeLife<'l> for Scope<'l, T> {}

/// Simplify holding a scoped object when we don't care about the lifetime
/// Can only be implemented by [Scope]
#[doc(hidden)]
pub trait Scoped: scope_private::SealedScope {}

impl<'l, T> Scoped for Scope<'l, T> {}

mod scope_private {
    pub trait SealedScope {}

    impl<'l, T> SealedScope for super::Scope<'l, T> {}
}

#[doc(hidden)]
pub trait Captures<T> {}
impl<T, U> Captures<T> for U {}
