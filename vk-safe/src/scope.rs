/*!
(🚧 TODO, this is being re worked to center around [`tag!()`], see [`crate::vk::create_instance`] for example use, more doc updates needed)
Scopes for unique object handling

All dispatchable handles (e.g. Instance, Device, etc.) implement all commands as methods on
a special **scoped** version of the particular handle. This is used to ensure different
handles are distinct.

This is because the Vulkan API exposes a complex parent child hierarchy, and
it must be ensured that "child" are resources only used with their respective "parent" resource
or "sibling" resources. In this regard, it is necessary to ensure that different instances
of a particular handle are unique from one another.

Use the [`scope!`] macro to create scoped versions of handles.

## Example
```
use vk_safe::vk;

// vk::scope!(instance1);
// vk::scope!(instance2);

// TODO, scope!() is being replaced with tag!()
```
*/

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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

/** A scoped type

*This is an implementation detail and you are not intended to directly use this*.

Types which are only safe to use within a scope implement their methods through this wrapper, or more preferably,
through the Deref Target [`SecretScope`].

#### implementation details
This is just informative, and not to be relied upon, as it could change. `Scope` is `#[repr(transparent)]`, and is a
wrapper around `*mut T`, and a `PhantomData` type to hold an invariant lifetime. It represents data that is tied to a
specific region of code. The `Scope` is considered to own the data it points to.
*/
#[repr(transparent)]
pub struct Scope<'id, T> {
    scope_inner: T,
    _id: ScopeId<'id>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Scope<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.scope_inner.fmt(f)
    }
}

impl<'id, T> Scope<'id, T> {
    /// DO NOT USE
    ///
    /// use [`scope!`]
    #[doc(hidden)]
    pub unsafe fn new(to_scope: T, _bounds: &ScopeBounds<'id>) -> Self {
        Self {
            scope_inner: to_scope,
            _id: Default::default(),
        }
    }

    pub(crate) fn from_tag(to_scope: T, _tag: Tag<'id>) -> Self {
        Self {
            scope_inner: to_scope,
            _id: Default::default(),
        }
    }
}

impl<T> Deref for Scope<'_, T> {
    type Target = SecretScope<Self, T>;
    /// Deref to SecretScope to hide the lifetime
    fn deref(&self) -> &Self::Target {
        // this is safe because:
        // - SecretScope repr(transparent) wrapper for T
        // - Scope repr(transparent) wrapper for T
        // - &Scope -> &T -> &SecretScope
        unsafe { std::mem::transmute::<&Self, &SecretScope<Self, T>>(self) }
    }
}

impl<T> DerefMut for Scope<'_, T> {
    /// DerefMut to SecretScope to hide the lifetime if T is `Mutable`
    fn deref_mut(&mut self) -> &mut Self::Target {
        // see comments in Deref impl for safety comment
        unsafe { std::mem::transmute::<&mut Self, &mut SecretScope<Self, T>>(self) }
    }
}

/// How a type should be scoped
///
/// Allow a type determine the proper way to scope it. Handle types
/// generally need to be Scoped by taking ownership of the handle.
/// However, some types may have use for being scoped, but it may
/// be sufficient to only use a borrow. This trait can be implemented
/// by types to allow taking ownership or borrowing accordingly.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "A Scope cannot be created with {Self}",
    label = "{Self} used here does not support being scoped",
    note = "`ToScope` is an implementation detail and is not public",
    note = "This is likely caused by using `scope!()` with a type defined outside of vk_safe",
    note = "Refer to vk_safe documentation for types that can/should be scoped"
)]
pub trait ToScope: Sized {
    unsafe fn to_scope<'id>(to_scope: Self, bounds: &ScopeBounds<'id>) -> Scope<'id, Self> {
        Scope::new(to_scope, bounds)
    }
}

/// Base trait for Handle traits
///
/// Handle traits like [`Instance`](crate::vk::Instance) are generally sub traits
/// of this trait.
pub trait HandleScope<T>: Deref<Target = SecretScope<Self, T>> + Sized {}

impl<T> HandleScope<T> for Scope<'_, T> {}

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
        #[allow(unused_mut)]
        let mut $name = unsafe { $crate::scope::ToScope::to_scope($name, &bounds) };
    };
}
#[doc(inline)]
pub use scope;

/// create a scope tag
///
/// Creates a [`Tag<'id>`](Tag) with the provided name.
///
/// The created [`Tag`] will be bound the the unique region of code
/// that it is defined in (i.e. a scope). Different invocations of [`tag!`] will
/// create different unique tags.
#[macro_export]
#[doc(hidden)]
macro_rules! tag {
    ( $name:ident ) => {
        let $name = unsafe { $crate::scope::Anchor::new() };
        let $name = unsafe { $crate::scope::ScopeBounds::new(&$name) };
        let $name = unsafe { $crate::scope::Tag::new(&$name) };
    };
}
#[doc(inline)]
pub use tag;

/** Deref target for Scope<'_, T>

*This is an implementation detail and you are not intended to directly use this*.

Types which are only safe to use within a scope implement their methods through this wrapper.

#### implementation details
This is just informative, and not to be relied upon, as it could change. `SecretScope` is a
`#[repr(transparent)]` wrapper around a `T: ?Sized`, and a `PhantomData<S>`. [`Scope<'_, T>`](Scope)
implements [`Deref`] with `Target = SecretScope<Self, T>`. In this way,
`SecretScope` can only ever exist as a reference bound to the lifetime of a [`Scope`], and
the invariant lifetime information is captured in the generic `S` type parameter.

vk-safe APIs can ensure different handles have the same scope (i.e. have the same Instance
or Device parent handle) by using the same generic parameter `S`.

The main reason to use this instead of [`Scope`] directly is to allow a "handle trait" pattern (See
modules in [`dispatchable_handles`](crate::dispatchable_handles)), and [`HandleScope`]. The handle
traits have Deref with the scoped handle type as the Target. This allows the handle traits to
abstract the `scopedness` of the handles while being transparently usable as the concrete type.
`Scope` cannot be directly used because it has a lifetime, and we cannot write e.g.
`trait Handle: Deref<Target = Scope<'scope, ConcreteHandle>>` because `'scope` is not defined.
We cannot even use `for<'scope> Deref<Target = Scope<'scope, ConcreteHandle>>` because we get
`error[E0582]: binding for associated type 'Target' references lifetime which does not appear in
the trait input types`.

`SecretScope` hides the lifetime by using `Scope` as a generic type parameter. We can then write
e.g. `trait Handle: Deref<Target = SecretScope<Self, ConcreteHandle>>`, where `Self` will be
`Scope<'_, ConcreteHandle>` as the implementor of the trait. Auto-deref then lets us seamlessly
call the concrete handle methods; e.g. given
`fn mf_fn(instance: impl Instance<Context: VERSION_1_0>) { let pds = instance.enumerate_physical_devices(Vec::new()); }`,
the `Instance` trait implies `Deref` to some concrete instance type, which has an `enumerate_physical_devices`
method (see details about context and version in [`context`](crate::context)).

# Safety
`SecretScope` is VERY delicate. It is only ever sound to have an a SecretScope which is
created by dereferencing `Scope`. In this way, for some handle `T` and lifetime `'scope`,
the concrete type will ALWAYS be `SecretScope<Scope<'scope, T>, T>`, even though
`Scope<'scope, T>` is abstracted away as a generic parameter `S`.
 */
#[repr(transparent)]
pub struct SecretScope<S, T> {
    scope: PhantomData<S>,
    inner: T,
}

impl<S, T> SecretScope<S, T> {
    /// Get back a reference to the `Scope`
    pub(crate) fn scope_ref(&self) -> &S {
        // We know that a valid SecretScope is ALWAYS constructed with S = Scope<'_, T>
        // We know &Scope and &SecretScope can be transmuted between each other
        // (see Deref for Scope)
        // thus, we know we can transmute to &S which should be &Scope
        unsafe { std::mem::transmute::<&Self, &S>(self) }
    }
}

impl<S, T> SecretScope<S, T> {
    /// manually get Deref target
    ///
    /// this is helpful because rust-analyzer seems to have trouble with autocompletion
    /// in some more complex uses of SecretScope, when going through auto-deref
    pub(crate) fn deref(&self) -> &T {
        &self.inner
    }
}

impl<S, T> std::ops::Deref for SecretScope<S, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
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
