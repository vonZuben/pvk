use std::future::Future;
use std::marker::PhantomData;
use std::borrow::Borrow;

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
pub struct Scope<'scope, H>{
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
        Self { handle, _id: Default::default() }
    }
}

impl<H> std::ops::Deref for Scope<'_, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        self.handle
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
pub trait ScopedAsyncFn<'scope, H: 'scope, O> : FnOnce(Scope<'scope, H>) -> Self::Future {
    type Future: Future<Output = O> + 'scope;
}
impl<'scope, H: 'scope, A, F, O> ScopedAsyncFn<'scope, H, O> for A
where
    A: FnOnce(Scope<'scope, H>) -> F,
    F: Future<Output = O> + 'scope
{
    type Future = F;
}

/// Create a task scope for a give T, by passing a scoped T to a given function or closure
pub fn scope<'a, F, R, T>(this: impl Borrow<T> + 'a, f: F) -> impl FnOnce() -> R + 'a
where
    for<'scope> F: FnOnce(Scope<'scope, T>) -> R + 'a,
    T: 'a
{
    move || f(Scope::new_scope(this.borrow()))
}

/// Create an async task scope for a given T, by passing a scoped T to a given async function or closure
///
/// #Note
/// this is currently limited by rust compiler because of lifetime issues with closures
/// 1) Higher ranked trait bound (used for scope lifetime) implies closure needs to live for static
/// 2) I have been able to change the trait bounds to avoid the static issue, but then the compiler appears to have
///     a hard time determining appropriate lifetime bounds for the Future returned by the closure.
pub fn async_scope<'a, A, R, T>(this: impl Borrow<T> + 'a, a: A) -> impl Future<Output = R> + 'a
where
    for<'scope> A: ScopedAsyncFn<'scope, T, R> + 'a,
    T: 'a
{
    async move { a(Scope::new_scope(this.borrow())).await }
}

//===============================
// traits to abstract over scoped objects

/// used to get the lifetime of a scoped object back for generic impls
pub trait ScopeLife<'l> : Scoped {}

impl<'l, T> ScopeLife<'l> for Scope<'l, T> {}

/// Simplify holding a scoped object when we don't care about the lifetime
pub trait Scoped { }

impl<'l, T> Scoped for Scope<'l, T> {}