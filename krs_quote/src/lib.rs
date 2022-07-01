//! Alternate quote macro (Not really intended for general use)
//!
//! This crate provides the [my_quote!] macro. It is a lot like [quote](https://docs.rs/quote/latest/quote/), but with different design decisions.
//!
//! 1) different syntax to allow much simpler `macro_rules!` implementation.
//! 2) I was annoyed that I couldn't use [IntoIterator] types, so this crate lets you do that.
//! 3) I only need to output to file, so this basically only generates a `String` stream.
//!
//! I also kind of only made it just for fun and to see what I could put together.
//!
//! Also, the string output inserts `\n` in specific places so that the output of any generated `macro_rules!` code looks nicer (since rustfmt can;t help there),
//! otherwise, I was getting files with every long single line `macro_rules!`.

#![warn(missing_docs)]

#[doc(hidden)]
pub use krs_hlist::{ Cons, End, higher_order::prelude::* };

mod to_tokens;
mod runtime;

pub use to_tokens::*;
pub use runtime::*;

/// The whole point!
///
/// Performs variable interpolation against the input and produces it as [TokenStream](to_tokens::TokenStream)
/// (which is basically just a `Vec<String>` for now).
#[macro_export]
macro_rules! my_quote {
    ( $($tt:tt)* ) => {{
        use $crate::ForEach;
        use $crate::prepare_different_types::*;
        let mut ts = $crate::TokenStream::new();
        let to_tokens = $crate::End$(.append($crate::tokenizer!($tt)))*;
        let mut ti = to_tokens.for_each($crate::ApplyPrepareQuote);
        ti.next().unwrap().for_each($crate::ApplyToTokens(&mut ts));
        ts
    }}
}

#[doc(hidden)]
#[macro_export]
macro_rules! tokenizer {

    // expand repetition wth separator
    ( {@$sep:tt* $($tt:tt)* } ) => {{
        use $crate::{HasIter, NoIter, FoldHasIter, FoldRef};
        let to_tokens = $crate::End$(.append($crate::tokenizer!($tt)))*;
        let _: HasIter = to_tokens.fold_ref(NoIter, FoldHasIter);
        match stringify!($sep) {
            "," => $crate::InnerRepWithSeparator::new(to_tokens, $crate::Comma.into()),
            ";" => $crate::InnerRepWithSeparator::new(to_tokens, $crate::SemiColon.into()),
            x => $crate::InnerRepWithSeparator::new(to_tokens, $crate::RawToken(x)),
        }
    }};

    // expand repetition
    ( {@* $($tt:tt)* } ) => {{
        use $crate::{HasIter, NoIter, FoldHasIter, FoldRef};
        let to_tokens = $crate::End$(.append($crate::tokenizer!($tt)))*;
        let _: HasIter = to_tokens.fold_ref(NoIter, FoldHasIter);
        $crate::InnerRep::new(to_tokens)
    }};

    // expand token
    ( {@$item:ident} ) => {{
        (&$item).as_to_prepare()
    }};

    // extract braces
    ( { $($tt:tt)* } ) => {{
        let to_tokens = $crate::End
            .append($crate::LeftBrace.as_to_prepare())
            $(.append($crate::tokenizer!($tt)))*
            .append($crate::RightBrace.as_to_prepare());
        $crate::HlistWrapper::new(to_tokens)
    }};

    // extract parens
    ( ( $($tt:tt)* ) ) => {{
        let to_tokens = $crate::End
            .append($crate::RawToken("("))
            $(.append($crate::tokenizer!($tt)))*
            .append($crate::RawToken(")"));
        $crate::HlistWrapper::new(to_tokens)
    }};

    // extract bracket
    ( [ $($tt:tt)* ] ) => {{
        let to_tokens = $crate::End
            .append($crate::RawToken("["))
            $(.append($crate::tokenizer!($tt)))*
            .append($crate::RawToken("]"));
        $crate::HlistWrapper::new(to_tokens)
    }};

    // special case fo comma
    ( , ) => {{
        $crate::Comma.as_to_prepare()
    }};

    // special case fo semicolon
    ( ; ) => {{
        $crate::SemiColon.as_to_prepare()
    }};

    // Regular token
    ( $tt:tt ) => {{
        $crate::RawToken(stringify!($tt)).as_to_prepare()
    }};

}

#[cfg(test)]
mod my_quote_test {

    use super::Token;

    #[test]
    fn make_token() {
        println!("=========make_token_test============");
        let t: Token = "hey".into();
        println!("{}", t);
    }

    #[test]
    fn use_my_quote() {
        println!("=========use_my_quote_test============");
        let s = "hello".to_string();
        let s2 = "me";
        let q = my_quote!(hey {@s} there {@s2});
        println!("{}", q);
    }

    #[test]
    fn use_my_quote_repeat() {
        println!("=========use_my_quote_repeat_test============");
        let friend = ["bill", "bob", "dave"];
        let greeting = ["hey", "welcome", "not you"];
        let bye = "and good bye";
        let q = my_quote!(greetings {@* {@greeting} {@friend} {@bye} } finally);
        println!("{}", q);
    }

    #[test]
    fn use_my_quote_repeat_with_separator() {
        println!("=========use_my_quote_repeat_with_separator============");
        let name = ["A", "B", "C", "D"];
        let q = my_quote!({@,* {@name} });
        println!("{}", q);
    }

    #[test]
    fn with_map() {
        println!("=========with_map============");
        let v = vec![1, 2, 3];
        let m = v.iter().map(|x|x+1);
        let q = my_quote!({@,* {@m} });
        println!("{}", q);
    }

    #[test]
    fn with_slice() {
        println!("=========with_slice============");
        let v = vec![1, 2, 3];
        let s = v.as_slice();
        let q = my_quote!({@,* {@s} });
        println!("{}", q);
    }
}
