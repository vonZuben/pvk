use std::future::Future;
use std::marker::PhantomData;

/// This represents an ID for a scope based on an invariant lifetime
///
/// by creating this and passing it to a closure with Higher-Rank Trait Bound lifetime (i.e. for<'scope>)
/// Id<'scope> will uniquely mark the specific scope of the closure body
///
/// this can be used to ensure different things should have the same Id
#[derive(Default, Clone, Copy)]
pub struct ScopeId<'id>(PhantomData<*mut &'id ()>);

impl ScopeId<'_> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

/// Wrapper for a reference to a type that should be limited to a specific scope marked by Id
///
/// types which are only safe within a scope implement their methods through this wrapper
/// (e.g. impl Scope<'id, Instance> {fn enumerate_physical_devices()})
#[repr(transparent)]
pub struct Scope<'scope, H> {
    handle: &'scope H,
    _id: ScopeId<'scope>,
}

impl<H> Clone for Scope<'_, H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<H> Copy for Scope<'_, H> {}

impl<H: std::fmt::Debug> std::fmt::Debug for Scope<'_, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'id, H> Scope<'id, H> {
    pub(crate) fn new_scope(handle: &'id H) -> Self {
        Self {
            handle,
            _id: Default::default(),
        }
    }
}

impl<H> std::ops::Deref for Scope<'_, H> {
    type Target = RefScope<Self, H>;
    fn deref(&self) -> &Self::Target {
        // this is safe because:
        // - Scope and RefScope are repr(transparent)
        // - Scope is a &'_ H
        // - RefScope is *const H
        // - &'_ H and *const H have same memory layout
        unsafe { std::mem::transmute::<&Self, &RefScope<Self, H>>(self) }
    }
}

/// Deref target for Scope<'_, T>
///
/// This type will be internally constructed to be RefScope<Scope<'scope, T>, T>
/// this is a trick to make the lifetime parameter vanish, while still keeping the lifetime information
///
/// I currently strongly believe that this should be completely sound. RefScope should essentially be equivalent to
/// a &'scope T. However, there may be some issue regarding stacked borrows as noted below?
///
/// At this time, according to https://doc.rust-lang.org/std/ptr/ based on https://github.com/rust-lang/rust/pull/103996
/// "The result of casting a reference to a pointer is valid for as long as the underlying object is live and no
/// reference (just raw pointers) is used to access the same memory. That is, reference and pointer accesses
/// cannot be interleaved."
///
/// In this regard, I consider the combination of Scope, RefScope, and Deref means there will certainly be interleaved use
/// of references and pointers to the same underlying object. But if RefScope is a glorified &'scope T,
/// then I do not understand the problem.
///
/// see also https://github.com/rust-lang/unsafe-code-guidelines/issues/463
///
/// Note: I am not so sure what is meant by "access". Since the RefScope pointer is never used to read/write, and is only used
/// to construct new references by Deref, then maybe there is no issue either way.
#[repr(transparent)]
pub struct RefScope<S, T> {
    handle: *const T,
    _scope: PhantomData<S>,
}

impl<S, H> RefScope<S, H> {
    pub(crate) fn to_scope(&self) -> S {
        // this is safe because
        // - we only construct RefScope as RefScope<Scope<'scope, H>, H>
        // - Scope and RefScope are compatible types as discussed in Deref for Scope
        // - thus, this is just reverting back to the original Scope
        unsafe { std::mem::transmute_copy(self) }
    }

    pub(crate) fn inner(&self) -> &H {
        &**self
    }
}

impl<S, H> std::ops::Deref for RefScope<S, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        // this is safe because:
        // - we only construct RefScope from Deref of Scope
        // - the S type parameter captures the lifetime information of the Scope
        // - thus, life of Scope >= RefScope<Scope, H>
        // - life of H >= Scope >= RefScope
        // - thus, we can create a &H with the shorter life of RefScope (from elided lifetime)
        unsafe { &*self.handle }
    }
}

impl<S, T> Clone for RefScope<S, T> {
    fn clone(&self) -> Self {
        RefScope {
            handle: self.handle,
            _scope: PhantomData,
        }
    }
}

impl<S, T> Copy for RefScope<S, T> {}

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

/// Create a task scope for a give T, by passing a scoped T to a given function or closure
pub fn scope<'a, F, R, T>(this: &'a T, f: F) -> impl FnOnce() -> R + 'a
where
    for<'scope> F: FnOnce(Scope<'scope, T>) -> R + 'a,
{
    move || f(Scope::new_scope(this))
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
