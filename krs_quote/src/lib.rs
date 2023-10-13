//! Alternate quote macro (Not really intended for general use)
//!
//! This crate provides the [krs_quote!] macro. It is a lot like [quote](https://docs.rs/quote/latest/quote/), but with different design decisions.
//! [krs_quote_with!] is provided to allow efficiently appending tokens to an already existing [TokenStream] (recommended to use in [ToTokens] implementations)
//!
//! 1) different syntax to allow much simpler `macro_rules!` implementation.
//! 2) I was annoyed that I couldn't use [IntoIterator] types, so this crate lets you do that.
//! 3) I only need to output to file, so this basically only generates a `String` stream. (kind of just a glorified format macro)
//!
//! I also kind of only made it just for fun and to see what I could put together.
//!
//! Also, the string output inserts `\n` in specific places so that the output of any generated `macro_rules!` code looks nicer (since rustfmt can't help there),
//! otherwise, I was getting files with very long single line `macro_rules!`.
//!
//! # Example
//! ```
//! use krs_quote::krs_quote;
//!
//! fn main() {
//!     let greeting = "Hello";
//!     let names = ["Foo", "Bar", "Zap"];
//!     let q = krs_quote!{
//!         {@* println!("{} {}", {@greeting}, {@names}); }
//!     };
//!     println!("{q}");
//! }
//! ```
//!
//! ## internal details
//!
//! This macro works by converting each user input into an iterator over `ToTokens` implementors. (see runtime/user_input.rs), and
//! creating an hlist tree comprising ToTokens iterators and other hlists.
//!
//! A single hlist represents a sequence of tokens that may be repeated any number of times (depending on what iterators are provided).
//! The outermost hlist is intended to only be used to produce one sequence of tokens (handled by `krs_quote!` and `krs_quote_with!`)
//! An inner hlist is intended to produce a sequence of tokens repeatedly based on user provided iterators (handled by the 'InnerRep'
//! and 'InnerRepWithSeparator' wrappers).
//!
//! Repetition is achieved by cloning the iterators. We try to only use iterators that "should" be cheap to clone (e.g.
//! iterators over references such that the whole collection is not closed).
//!
//! 'MaybeCloneTokenIter' is an internal detail that avoids recursive cloning of hlists.
//!
//! 'TokenIter' is another internal detail to avoid cloning inner hlists.

#![warn(missing_docs)]

mod runtime;
mod to_tokens;

pub use to_tokens::{ToTokens, ToTokensClosure, Token, TokenStream};

#[doc(hidden)]
pub mod __private {
    pub use super::runtime::*;
    pub use super::to_tokens::*;

    pub fn coerce<'a, const N: usize>(
        a: [Box<dyn GenerateTokens + 'a>; N],
    ) -> [Box<dyn GenerateTokens + 'a>; N] {
        a
    }
}

/// The whole point!
///
/// Performs variable interpolation against the input and produces it as [TokenStream](to_tokens::TokenStream)
/// (which is basically just a `Vec<String>` for now).
#[macro_export]
macro_rules! krs_quote {
    ( $($tt:tt)* ) => {{
        use $crate::__private::*;
        let mut ts = TokenStream::new();
        let mut to_tokens = coerce([$(Box::new($crate::quote_each_tt!($tt))),*]);
        to_tokens.init();
        to_tokens.advance_token();
        to_tokens.to_tokens(&mut ts);
        ts
    }}
}

/// krs_quote, but append tokens to an existing [TokenStream]
///
/// This should be preferred to use inside [ToTokens] implementations
///
/// # Example
/// ```
/// use krs_quote::{krs_quote_with, ToTokens, TokenStream};
///
/// struct CustomId(i32);
///
/// impl ToTokens for CustomId {
///     fn to_tokens(&self, tokens: &mut TokenStream) {
///         let id = format!("Id{}", self.0);
///         krs_quote_with!(tokens <- {@id} );
///     }
/// }
/// ```
#[macro_export]
macro_rules! krs_quote_with {
    ( $ts:ident <- $($tt:tt)* ) => {{
        use $crate::__private::*;
        let ts: &mut TokenStream = $ts;
        let mut to_tokens = coerce([$(Box::new($crate::quote_each_tt!($tt))),*]);
        to_tokens.init();
        to_tokens.advance_token();
        to_tokens.to_tokens(ts);
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! quote_each_tt {

    // expand repetition with comma
    ( {@,* $($tt:tt)* } ) => {{
        Repeat(coerce([
            Box::new(SkipFirst::new(Comma)),
            $(Box::new($crate::quote_each_tt!($tt))),*
        ]))
    }};

    // expand repetition with semi colon
    ( {@;* $($tt:tt)* } ) => {{
        Repeat(coerce([
            Box::new(SkipFirst::new(SemiColon)),
            $(Box::new($crate::quote_each_tt!($tt))),*
        ]))
    }};

    // expand repetition with any separator
    ( {@$sep:tt* $($tt:tt)* } ) => {{
        Repeat(coerce([
            Box::new(SkipFirst::new(RawToken($sep))),
            $(Box::new($crate::quote_each_tt!($tt))),*
        ]))
    }};

    // expand repetition
    ( {@* $($tt:tt)* } ) => {{
        Repeat(coerce([$(Box::new($crate::quote_each_tt!($tt))),*]))
    }};

    // expand token
    ( {@$item:ident} ) => {{
        TokenAdvancer::new((&$item).input())
    }};

    // extract braces
    ( { $($tt:tt)* } ) => {{
        coerce([
            Box::new(TokenAdvancer::new(LeftBrace.input())),
            $(Box::new($crate::quote_each_tt!($tt)),)*
            Box::new(TokenAdvancer::new(RightBrace.input())),
        ])
    }};

    // extract parens
    ( ( $($tt:tt)* ) ) => {{
        coerce([
            Box::new(TokenAdvancer::new(RawToken("(").input())),
            $(Box::new($crate::quote_each_tt!($tt)),)*
            Box::new(TokenAdvancer::new(RawToken(")").input())),
        ])
    }};

    // extract bracket
    ( [ $($tt:tt)* ] ) => {{
        coerce([
            Box::new(TokenAdvancer::new(RawToken("[").input())),
            $(Box::new($crate::quote_each_tt!($tt)),)*
            Box::new(TokenAdvancer::new(RawToken("]").input())),
        ])
    }};

    // special case fo comma
    ( , ) => {{
        TokenAdvancer::new(Comma.input())
    }};

    // special case fo semicolon
    ( ; ) => {{
        TokenAdvancer::new(SemiColon.input())
    }};

    // Regular token
    ( $tt:tt ) => {{
        TokenAdvancer::new(RawToken(stringify!($tt)).input())
    }};

}

#[cfg(test)]
mod krs_quote_test {

    use super::Token;

    #[test]
    fn make_token() {
        println!("=========make_token_test============");
        let t: Token = "hey".into();
        println!("{}", t);
    }

    #[test]
    fn use_krs_quote() {
        println!("=========use_krs_quote_test============");
        let s = "hello".to_string();
        let s2 = "me";
        let q = krs_quote!(hey {@s} there {@s2});
        println!("{}", q);
    }

    #[test]
    fn use_krs_quote_repeat() {
        println!("=========use_krs_quote_repeat_test============");
        let friend = ["bill", "bob", "dave"];
        let greeting = ["hey", "welcome", "not you"];
        let bye = "and good bye";
        let q = krs_quote!(greetings {@* {@greeting} {@friend} {@bye} } finally);
        println!("{}", q);
    }

    #[test]
    fn use_krs_quote_repeat_with_separator() {
        println!("=========use_krs_quote_repeat_with_separator============");
        let name = ["A", "B", "C", "D"];
        let q = krs_quote!({@,* {@name} });
        println!("{}", q);
    }

    #[test]
    fn with_map() {
        println!("=========with_map============");
        let v = vec![1, 2, 3];
        let m = v.iter().map(|x| x + 1);
        let q = krs_quote!({@,* {@m} });
        println!("{}", q);
    }

    #[test]
    fn with_slice() {
        println!("=========with_slice============");
        let v = vec![1, 2, 3];
        let s = v.as_slice();
        let q = krs_quote!({@,* {@s} });
        println!("{}", q);
    }

    #[test]
    fn tmp() {
        println!("=========empty_quote============");
        let _q = krs_quote!();
    }
}
