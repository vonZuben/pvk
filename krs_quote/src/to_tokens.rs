use std::fmt;
use std::rc::Rc;

use crate::runtime::GetTokenIter;

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

impl ToTokens for bool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            true => tokens.push(Token::from("true")),
            false => tokens.push(Token::from("false")),
        }
    }
}

impl<T: ToTokens + ?Sized> ToTokens for Box<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        <T as ToTokens>::to_tokens(self, tokens)
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

impl ToTokens for () {
    fn to_tokens(&self, _tokens: &mut TokenStream) {}
}

/// Wrap a closure that can impl ToTokens
///
/// sometimes it is handy to define a closure to generate tokens into a TokenStream
pub struct ToTokensClosure<F>(pub F);

impl<F: Fn(&mut TokenStream)> ToTokens for ToTokensClosure<F> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0(tokens)
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

/// The output of [krs_quote!](crate::krs_quote!)
pub struct TokenStream(Vec<Token>);

impl TokenStream {
    /// Not really intended for use
    ///
    /// used automatically my [krs_quote!](crate::krs_quote!)
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

// Some(()) signals something was generated, and None means nothing was generated
type Signal = Option<()>;

#[doc(hidden)]
pub struct TokenAdvancer<S, I, T> {
    source: S,
    iter: Option<I>,
    to_tokens: Option<T>,
}

impl<S: GetTokenIter> TokenAdvancer<S, S::TokenIter, S::Item> {
    pub fn new(source: S) -> Self {
        Self { source, iter: None, to_tokens: None }
    }
}

#[doc(hidden)]
pub trait GenerateTokens {
    fn init(&mut self);
    fn advance_token(&mut self) -> Signal;
    fn to_tokens(&mut self, ts: &mut TokenStream);
}

impl<S: GetTokenIter> GenerateTokens for TokenAdvancer<S, S::TokenIter, S::Item> {
    fn init(&mut self) {
        self.iter = Some(self.source.get_token_iter());
    }

    fn advance_token(&mut self) -> Signal {
        self.to_tokens = Some(self.iter.as_mut().unwrap().next()?); // although next() already returns Option, we need to know if it was some or not for the Signal, so we also re-wrap in Some(_)
        Some(())
    }

    fn to_tokens(&mut self, ts: &mut TokenStream) {
        self.to_tokens.as_ref().unwrap().to_tokens(ts);
    }
}

impl<'a, const N: usize> GenerateTokens for [Box<dyn GenerateTokens + 'a>; N] {
    fn init(&mut self) {
        for x in self.iter_mut() {
            x.init();
        }
    }

    fn advance_token(&mut self) -> Signal {
        for x in self.iter_mut() {
            x.advance_token()?;
        }
        Some(())
    }

    fn to_tokens(&mut self, ts: &mut TokenStream) {
        for x in self.iter_mut() {
            x.to_tokens(ts);
        }
    }
}

enum Skip {
    Init,
    Skip,
    Ok,
}

#[doc(hidden)]
pub struct SkipFirst<T> {
    to_tokens: T,
    skip: Skip,
}

impl<T> SkipFirst<T> {
    pub fn new(t: T) -> Self {
        Self { to_tokens: t, skip: Skip::Init }
    }
}

impl<T: ToTokens> GenerateTokens for SkipFirst<T> {
    fn init(&mut self) {
        self.skip = Skip::Init;
    }

    fn advance_token(&mut self) -> Signal {
        match self.skip {
            Skip::Init => self.skip = Skip::Skip,
            Skip::Skip => self.skip = Skip::Ok,
            Skip::Ok => {}
        };
        Some(())
    }

    fn to_tokens(&mut self, ts: &mut TokenStream) {
        match self.skip {
            Skip::Ok => self.to_tokens.to_tokens(ts),
            _ => {}
        }
    }
}

enum OneState {
    One,
    Done,
}

#[doc(hidden)]
pub struct One<T> {
    to_tokens: T,
    state: OneState,
}

impl<T> One<T> {
    pub fn new(t: T) -> Self {
        One { to_tokens: t, state: OneState::One }
    }
}

impl<T: ToTokens> GenerateTokens for One<T> {
    fn init(&mut self) {
        self.state = OneState::One;
    }

    fn advance_token(&mut self) -> Signal {
        match self.state {
            OneState::One => {
                self.state = OneState::Done;
                Some(())
            }
            OneState::Done => {
                None
            }
        }
    }

    fn to_tokens(&mut self, ts: &mut TokenStream) {
        self.to_tokens.to_tokens(ts);
    }
}

#[doc(hidden)]
pub struct Repeat<R>(pub R);

impl<R: GenerateTokens> GenerateTokens for Repeat<R> {
    fn init(&mut self) {}

    fn advance_token(&mut self) -> Signal { Some(()) }

    fn to_tokens(&mut self, ts: &mut TokenStream) {
        self.0.init();
        while self.0.advance_token().is_some() {
            self.0.to_tokens(ts);
        }
    }
}