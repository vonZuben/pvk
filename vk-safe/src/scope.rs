/*!
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

use std::cell::UnsafeCell;
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

/// Indicate that is it ok to create a [`Shared`] with the implementor type
///
/// This **MUST NOT** be implemented at the same time as [`Mutable`]
#[doc(hidden)]
pub unsafe trait Shareable {}

/// Indicate that is it ok to DerefMut with the Scoped implementor type
///
/// This **MUST NOT** be implemented at the same time as [`Shareable`]
#[doc(hidden)]
pub unsafe trait Mutable {}

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
through the Deref Target [`SecretScope`].

#### implementation details
This is just informative, and not to be relied upon, as it could change. `Scope` is `#[repr(transparent)]`, and is a
wrapper around `*mut T`, and a `PhantomData` type to hold an invariant lifetime. It represents data that is tied to a
specific region of code. The `Scope` is considered to own the data it points to.
*/
#[repr(transparent)]
pub struct Scope<'id, T> {
    scope_inner: *mut T,
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
    pub unsafe fn new(to_scope: &'id mut UnsafeCell<T>, _bounds: &ScopeBounds<'id>) -> Self {
        Self {
            scope_inner: to_scope.get(),
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
        // - Scope repr(transparent) wrapper for *mut T
        // - Scope -> *mut T -> &T -> &SecretScope
        unsafe { std::mem::transmute::<*mut T, &SecretScope<Self, T>>(self.scope_inner) }
    }
}

impl<T: Mutable> DerefMut for Scope<'_, T> {
    /// DerefMut to SecretScope to hide the lifetime if T is `Mutable`
    ///
    /// `Mutable` is an internal trait that is implemented for handle types
    /// that do not need to be shared and can have methods that take `&mut Self`
    fn deref_mut(&mut self) -> &mut Self::Target {
        // see comments in Deref impl for safety comment
        unsafe { std::mem::transmute::<*mut T, &mut SecretScope<Self, T>>(self.scope_inner) }
    }
}

/// Provide only immutable access to Scope
///
/// This is used when child handles need access to the parent handle
/// to ensure that no mutable methods can be accessed.
///
/// # Safety
/// This type does **NOT** borrow the `Scope`, and only holds
/// a pointer to the data that is alive for 'scope. While, the data
/// is guaranteed to be alive, it should be assumed to always be
/// aliased. Thus, no safe api relying on &mut access can be created
/// for `Scope` or `SecretScope` if `Shared` will exist.
///
/// A type may be [`Shareable] XOR [`Mutable`] to allow creation
/// of [`Shared`] XOR `&mut api()`.
///
/// ## implementation detail
/// This type only holds a mutable pointer to unit type and a
/// PhantomData to S. We can assume that S is always some
/// `Scope<'_, T>` because it is only created frm a `SecretScope`
/// which can only be created from a `Scope`. Thus, we can
/// deref to S by transmuting the inner pointer to a `Scope`
/// since the layout is the same.
#[doc(hidden)]
#[repr(transparent)]
pub struct Shared<S> {
    scope_layout: *mut (),
    scope: PhantomData<S>,
}

impl<S> Shared<S> {
    unsafe fn new<T: ?Sized + Shareable>(ss: &SecretScope<S, T>) -> Self {
        // Here, we want to reverse the Deref of Scope<'_, T> such as by transmute (see Deref for Scope).
        // However, the compiler cannot know that S is correctly sized for all S,
        // but WE know that S is ALWAYS `Scope<'_, T>` (or else it was improperly constructed).
        // Thus, we can use transmute_copy.
        Shared {
            scope_layout: unsafe { std::mem::transmute_copy::<&SecretScope<S, T>, *mut ()>(&ss) },
            scope: PhantomData,
        }
    }
}

impl<S> Clone for Shared<S> {
    fn clone(&self) -> Self {
        Self {
            scope_layout: self.scope_layout.clone(),
            scope: self.scope.clone(),
        }
    }
}

impl<S> Copy for Shared<S> {}

impl<S> Deref for Shared<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute::<&*mut (), &S>(&self.scope_layout) }
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
        let mut cell = std::cell::UnsafeCell::new($name);
        let bounds = unsafe { $crate::scope::ScopeBounds::new(&anchor) };
        #[allow(unused_mut)]
        let mut $name = unsafe { $crate::scope::Scope::new(&mut cell, &bounds) };
    };
}
#[doc(inline)]
pub use scope;

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
pub struct SecretScope<S, T: ?Sized> {
    scope: PhantomData<S>,
    inner: T,
}

impl<S, T: ?Sized + Shareable> SecretScope<S, T> {
    /// Get a Shared Scope
    pub(crate) fn shared(&self) -> Shared<S> {
        unsafe { Shared::new(self) }
    }
}

impl<S, T: ?Sized> SecretScope<S, T> {
    /// manually get Deref target
    ///
    /// this is helpful because rust-analyzer seems to have trouble with autocompletion
    /// in some more complex uses of SecretScope, when going through auto-deref
    pub(crate) fn deref(&self) -> &T {
        &self.inner
    }
}

impl<S, T: ?Sized> std::ops::Deref for SecretScope<S, T> {
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

#[doc(hidden)]
pub trait Captures<T> {}
impl<T, U> Captures<T> for U {}
