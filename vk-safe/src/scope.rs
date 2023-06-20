use std::future::Future;
use std::marker::PhantomData;

/// This represents an ID for a scope based on an invariant lifetime
///
/// by creating this and passing it to a closure with Higher-Rank Trait Bound lifetime (i.e. for<'scope>)
/// Id<'scope> will uniquely mark the specific scope of the closure body
///
/// this can be used to ensure different things should have the same Id
#[derive(Default)]
pub struct ScopeId<'id>(PhantomData<*mut &'id ()>);

/// Wrapper for a type that should be limited to a specific scope marked by Id
///
/// types which are only safe within a scope implement their methods through this wrapper
/// (e.g. impl Scope<'id, Instance> {fn enumerate_physical_devices()})
pub struct ScopedHandle<'scope, H>{
    handle: H,
    _id: ScopeId<'scope>,
}

impl<H: std::fmt::Debug> std::fmt::Debug for ScopedHandle<'_, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl<'id, H> ScopedHandle<'id, H> {
    pub(crate) fn new_scope(handle: H) -> Self {
        Self { handle, _id: Default::default() }
    }
}

impl<H> std::ops::Deref for ScopedHandle<'_, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

/// Represents an async fn which produces a Future that is restricted to 'scope
/// but can produce an output O that is not limited to 'scope
///
/// O needs to be an input to the trait (rather than accessing from associate type Future::Output),
/// since O should not be generic over the 'scope
pub trait ScopedAsyncFn<'scope, H: 'scope, O> : FnOnce(ScopedHandle<'scope, &'scope H>) -> Self::Future {
    type Future: Future<Output = O> + 'scope;
}
impl<'scope, H, A, F, O> ScopedAsyncFn<'scope, H, O> for A
where
    A: FnOnce(ScopedHandle<'scope, &'scope H>) -> F,
    H: 'scope,
    F: Future<Output = O> + 'scope
{
    type Future = F;
}

/// Represent an owned handle (e.g. vk::Instance) with restricted capability
///
/// Most (if not all) functionality of handles is only safe when used within a limited scope
/// This type provides a way to create such safe scopes where the real functionality of the handles
/// can be used
#[derive(Debug)]
pub struct ProtectedHandle<H> {
    handle: H,
}

impl<H> ProtectedHandle<H> {
    pub(crate) fn new(handle: H) -> Self {
        Self { handle }
    }

    pub fn scoped_task<'a, F, R>(&'a self, f: F) -> impl FnOnce() -> R + 'a
    where
        for<'scope> F: FnOnce(ScopedHandle<'scope, &'scope H>) -> R + 'a,
    {
        move || f(ScopedHandle::new_scope(&self.handle))
    }

    ///
    pub fn scoped_async_task<'a, A, R>(&'a self, a: A) -> impl Future<Output = R> + 'a
    where
        for<'scope> A: ScopedAsyncFn<'scope, H, R> + 'a,
    {
        async move { a(ScopedHandle::new_scope(&self.handle)).await }
    }
}
