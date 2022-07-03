use std::fmt;
use std::rc::Rc;

/// Single token for a [TokenStream]
#[derive(Clone)]
pub struct Token(Rc<String>);

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
    RightBrace => "\n}\n",
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

//impl Extend<Token> for TokenStream {
//    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=Token> {
//        self.0.extend(iter)
//    }
//}

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
