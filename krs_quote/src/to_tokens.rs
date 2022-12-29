use std::fmt;
use std::rc::Rc;

use krs_hlist::{Cons, End};

/// Single token for a [TokenStream]
#[derive(Clone)]
pub struct Token(Rc<String>);

impl Token {
    /// convert a string into a Token, preserving the stringness (rather than treating it like code)
    pub fn str_as_token(s: impl AsRef<str>) -> Self {
        format!("\"{}\"", s.as_ref()).into()
    }
}

impl From<&str> for Token {
    fn from(s: &str) -> Self {
        Self(s.to_string().into())
    }
}

impl From<String> for Token {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

/// produce tokens
///
/// # Example
/// ```
/// use krs_quote::{ToTokens, TokenStream};
///
/// struct A;
///
/// impl ToTokens for A {
///     fn to_tokens(&self, tokens: &mut TokenStream) {
///         tokens.push("A".to_string());
///     }
/// }
/// ```
pub trait ToTokens {
    /// produce tokens into a [TokenStream]
    fn to_tokens(&self, tokens: &mut TokenStream);
}

impl<T: ToTokens + ?Sized> ToTokens for &T {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        (**self).to_tokens(tokens)
    }
}

impl ToTokens for str {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push(format!("\"{}\"", self))
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push(format!("\"{}\"", self))
    }
}

impl ToTokens for Token {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push(self.clone())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Some(t) => t.to_tokens(tokens),
            None => {}
        }
    }
}

macro_rules! impl_to_tokens_for_numbers {
    ( $( $nt:ty ),* $(,)? ) => {
        $(
            impl ToTokens for $nt {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    tokens.push(format!("{}", self))
                }
            }
        )*
    }
}

impl_to_tokens_for_numbers!{
    i8, u8,
    i16, u16,
    i32, u32,
    i64, u64,
    i128, u128,
    f32, f64,
    usize, isize,
}

#[doc(hidden)]
/// special tokens for specific situations
pub trait SpecialToken {
    const TOKEN: RawToken;
}

macro_rules! make_special_token {
    ( $( $name:ident => $token:literal ),* $(,)? ) => {
        $(
            #[doc(hidden)]
            pub struct $name;
            impl SpecialToken for $name {
                const TOKEN: RawToken = RawToken($token);
            }
            impl ToTokens for $name {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    Self::TOKEN.to_tokens(tokens);
                }
            }
        )*
    };
}

make_special_token!{
    Comma => ",\n",
    SemiColon => ";\n",
    LeftBrace => "{\n",
    RightBrace => "}\n",
}

/// The output of [my_quote!]
pub struct TokenStream(Vec<Token>);

impl TokenStream {
    /// Not really intended for use
    ///
    /// used automatically my [my_quote!]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push a token into the steam
    pub fn push(&mut self, t: impl Into<Token>) {
        self.0.push(t.into());
    }
}

impl fmt::Display for TokenStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for tt in self.0.iter() {
            write!(f, "{} ", tt)?;
        }
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub struct RawToken(pub &'static str);

impl ToTokens for RawToken {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.push(self.0)
    }
}

impl<T: SpecialToken> From<T> for RawToken {
    fn from(_: T) -> Self {
        T::TOKEN
    }
}

impl<H: ToTokens, T: ToTokens> ToTokens for Cons<H, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.head.to_tokens(tokens);
        self.tail.to_tokens(tokens);
    }
}

impl ToTokens for End {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // End
    }
}

impl<H: TokenIter, T: TokenIter> TokenIter for Cons<H, T> {
    type Item<'a> = Cons<H::Item<'a>, T::Item<'a>> where H:'a, T:'a;

    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        Cons { head: self.head.next_token()?, tail: self.tail.next_token()? }.into()
    }
}

impl TokenIter for End {
    type Item<'a> = End;

    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        End.into()
    }
}

/// This manages a repeated sequence of token
#[doc(hidden)]
pub struct InnerRep<L>(L);

impl<L> InnerRep<L> {
    pub fn new(l: L) -> Self {
        Self(l)
    }
}

impl<L: MaybeCloneTokenIter> ToTokens for InnerRep<L> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut list_iter = self.0.maybe_clone_token_iter();
        while let Some(tt) = list_iter.next_token() {
            tt.to_tokens(tokens);
        }
    }
}

impl<L> TokenIter for InnerRep<L> where Self: ToTokens {
    type Item<'a> = &'a Self where Self: 'a;
    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        Some(self)
    }
}

impl<L> MaybeCloneTokenIter for InnerRep<L> where Self: TokenIter {
    type Item<'a> = &'a Self where Self: 'a;
    fn maybe_clone_token_iter<'a>(&'a self) -> Self::Item<'a> {
        self
    }
}

/// This manages a repeated sequence of token with a separator between each sequence
#[doc(hidden)]
pub struct InnerRepWithSeparator<L, S>(L, S);

impl<L, S> InnerRepWithSeparator<L, S> {
    pub fn new(l: L, s: S) -> Self {
        Self(l, s)
    }
}

impl<L: MaybeCloneTokenIter, S: ToTokens> ToTokens for InnerRepWithSeparator<L, S> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut list_iter = self.0.maybe_clone_token_iter();
        // add first item
        list_iter.next_token().map(|tt|tt.to_tokens(tokens));

        // if any more items, there is a separator first
        while let Some(tt) = list_iter.next_token() {
            self.1.to_tokens(tokens);
            tt.to_tokens(tokens);
        }
    }
}

impl<L, S> TokenIter for InnerRepWithSeparator<L, S> where Self: ToTokens {
    type Item<'a> = &'a Self where Self: 'a;
    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        Some(self)
    }
}

impl<L, S> MaybeCloneTokenIter for InnerRepWithSeparator<L, S> where Self: ToTokens {
    type Item<'a> = &'a Self where Self: 'a;
    fn maybe_clone_token_iter<'a>(&'a self) -> Self::Item<'a> {
        self
    }
}

/// This is just a streaming iterator which I need for the GAT
///
/// I just all it 'TokenIter' to represent how I am using it
/// just an internal implementation detail anyway
///
/// I need the internally repeated token sequences to be repeatable
/// as many times as needed, and i don't want to Clone just for lifetime constraints
///
/// essentially just a performance optimization
#[doc(hidden)]
pub trait TokenIter {
    type Item<'a>: ToTokens where Self: 'a;
    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}

/// This is just a hack to satisfy trait constraints on MaybeCloneTokenIter, since the Item needs to
/// impl 'TokenIter', but that is not normally the case for 'InnerRep', and 'InnerRepWithSeparator'
/// since they pass &Self. This provides the needed implementation but is never needed to be called.
impl<I: TokenIter> TokenIter for &I {
    type Item<'a> = I::Item<'a> where Self: 'a;

    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        panic!("This should never be called, and only exists to satisfy trait bounds that are needed due to current GAT limitations")
    }
}

/// This is for newtype pattern in order to avoid conflicting trait implementation issues
#[doc(hidden)]
#[derive(Clone)]
pub struct IterWrapper<I>(pub I);

impl<I: Iterator> TokenIter for IterWrapper<I> where I::Item: ToTokens {
    type Item<'a> = I::Item where Self: 'a;

    fn next_token<'a>(&'a mut self) -> Option<Self::Item<'a>> {
        self.0.next()
    }
}

/// This is for performance
///
/// the internal repetitions ('InnerRep' and 'InnerRepWithSeparator') don;t need to be completely cloned
/// so they implement this by just passing a reference to them selves
///
/// The 'Item' implements TokenIter because it is needed in the 'InnerRep' and 'InnerRepWithSeparator'
/// I would actually prefer if this trait was just 'MaybeClone' with not limitation on the item.
///
/// However, then 'InnerRep' and 'InnerRepWithSeparator' would need to express the 'TokenIter' requirement
/// via 'for<'a> <L as MaybeClone>::Item<'a>: TokenIter', which has a current GAT limitation of making 'a: 'static.
/// By expressing the trait bound I need here, I avoid the above issue.
#[doc(hidden)]
pub trait MaybeCloneTokenIter {
    type Item<'a>: TokenIter where Self: 'a;
    fn maybe_clone_token_iter<'a>(&'a self) -> Self::Item<'a>;
}

impl<C> MaybeCloneTokenIter for C
where
    C: Clone + TokenIter,
{
    type Item<'a> = Self where Self: 'a;

    fn maybe_clone_token_iter(&self) -> Self {
        self.clone()
    }
}