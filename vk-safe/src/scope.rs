use std::future::Future;
use std::marker::PhantomData;

/// This represents an ID for a scope based on an invariant lifetime
///
/// by creating this and passing it to a closure with Higher-Rank Trait Bound lifetime (i.e. for<'scope>)
/// Id<'scope> will uniquely mark the specific scope of the closure body
///
/// this can be used to ensure different things should have the same Id
#[derive(Default, Clone, Copy)]
#[repr(transparent)]
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
    pub(crate) fn new_scope(handle: &'id T) -> Self {
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

/// Represents an async fn which produces a Future that is restricted to 'scope
/// but can produce an output O that is not limited to 'scope
///
/// #Note
/// The main purpose of this is to represent for<'a> FnOnce(I) -> O where O: 'a
/// it is not possible to write where O: 'a in this way. It is also not possible to
/// restrict the output to 'a otherwise because we cannot name
/// the type of O which is an anonymous type implementing Future
pub trait ScopedAsyncFn<'scope, H: 'scope, O>: FnOnce(Scope<'scope, H>) -> Self::Future {
    type Future: Future<Output = O> + 'scope;
}
impl<'scope, H: 'scope, A, F, O> ScopedAsyncFn<'scope, H, O> for A
where
    A: FnOnce(Scope<'scope, H>) -> F,
    F: Future<Output = O> + 'scope,
{
    type Future = F;
}

/// Create a 'Scope'
///
/// Call this function with a handle that is to be scoped, and a function / closure that will use the scoped handle.
///
/// A scoped handle is mostly an implementation detail, but a user of a scoped handle needs to be aware of the limitations.
/// A scoped handle has an invariant lifetime (see [Subtyping and Variance](https://doc.rust-lang.org/nomicon/subtyping.html) to learn more).
/// The invariant lifetime is completely unique within the scope of the passed in function / closure, and it cannot be unified with other Scopes.
/// This allows us to ensure that different instances of handles are handles separately.
///
/// ℹ️ I recently found [generativity crate](https://docs.rs/generativity/latest/generativity/), and I am investigating if it is sound.
/// If it is sound, then I will consider depreciating this for a guard like api since it is easier to use.
pub fn scope<F, R, T>(this: T, f: F) -> impl FnOnce() -> R
where
    for<'scope> F: FnOnce(Scope<'scope, T>) -> R,
{
    move || f(Scope::new_scope(&this))
}

/// Create an async task scope for a given T, by passing a scoped T to a given async function or closure
///
/// #Note
/// this is currently limited by rust compiler because of lifetime issues with closures
/// 1) Higher ranked trait bound (used for scope lifetime) implies closure needs to live for static
/// 2) I have been able to change the trait bounds to avoid the static issue, but then the compiler appears to have
///     a hard time determining appropriate lifetime bounds for the Future returned by the closure.
pub fn async_scope<'a, A, R, T>(this: &'a T, a: A) -> impl Future<Output = R> + 'a
where
    for<'scope> A: ScopedAsyncFn<'scope, T, R> + 'a,
{
    async move { a(Scope::new_scope(this)).await }
}

//===============================
// traits to abstract over scoped objects

/// used to get the lifetime of a scoped object back for generic impls
pub trait ScopeLife<'l>: Scoped {}

impl<'l, T> ScopeLife<'l> for Scope<'l, T> {}

/// Simplify holding a scoped object when we don't care about the lifetime
/// Can only be implemented by [Scope]
pub trait Scoped: scope_private::SealedScope {}

impl<'l, T> Scoped for Scope<'l, T> {}

mod scope_private {
    pub trait SealedScope {}

    impl<'l, T> SealedScope for super::Scope<'l, T> {}
}

pub trait Captures<T> {}
impl<T, U> Captures<T> for U {}
