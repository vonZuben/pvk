use std::fmt;
use std::rc::Rc;

use std::marker::PhantomData;

pub use krs_hlist::{Cons, End, higher_order_prelude::* };

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


pub trait ToTokens {
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

trait SpecialToken {
    const TOKEN: RawToken;
}

macro_rules! make_special_token {
    ( $( $name:ident => $token:literal ),* $(,)? ) => {
        $(
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

pub struct ApplyToTokens<'t>(pub &'t mut TokenStream);

impl<'a, 't, T: ToTokens> FuncMut<&'a T> for ApplyToTokens<'t> {
    type Output = ();
    fn call_mut(&mut self, i: &'a T) {
        i.to_tokens(self.0)
    }
}

pub struct TokenStream(Vec<Token>);

impl TokenStream {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, t: impl Into<Token>) {
        self.0.push(t.into());
    }
}

impl Extend<Token> for TokenStream {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item=Token> {
        self.0.extend(iter)
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

#[derive(Copy, Clone, Debug)]
pub struct RawToken(pub &'static str);

impl PrepareQuote for RawToken {
    type Output = std::iter::Repeat<RawToken>;
    fn prepare_quote(self) -> Self::Output {
        std::iter::repeat(self.clone())
    }
}

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

pub trait PrepareQuote {
    type Output;
    fn prepare_quote(self) -> Self::Output;
}

impl<P: PrepareQuote + Copy> PrepareQuote for &P {
    type Output = P::Output;

    fn prepare_quote(self) -> Self::Output {
        (*self).prepare_quote()
    }
}

#[derive(Debug)]
pub struct ToPrepare<'a, T: ?Sized, KIND>(&'a T, PhantomData<KIND>);

impl<'a, T: ?Sized, KIND> Copy for ToPrepare<'a, T, KIND> {}

impl<'a, T: ?Sized, KIND> Clone for ToPrepare<'a, T, KIND> {
    fn clone(&self) -> Self {
        *self
    }
}

macro_rules! to_prepare_trait {
    ( $trait_name:ident -> $kind:ident <$life:lifetime, $ty:tt> for $apply:ty { where $($applicable:tt)* } {
        type Output = $out:ty;
        |$this:ident| $prepare:expr
    }) => {

        #[derive(Copy, Clone, Debug)]
        pub struct $kind;

        pub trait $trait_name<$life, $ty: ?Sized> {
            fn as_to_prepare(&$life self) -> ToPrepare<$life, $ty, $kind>;
        }

        impl<$life, $ty: ?Sized> $trait_name<$life, $ty> for $apply where $($applicable)* {
            fn as_to_prepare(&$life self) -> ToPrepare<$life, $ty, $kind> {
                ToPrepare(self, PhantomData)
            }
        }

        impl<$life, $ty: ?Sized> PrepareQuote for ToPrepare<$life, $ty, $kind> where $($applicable)* {
            type Output = $out;
            fn prepare_quote(self) -> Self::Output {
                (|$this: Self| $prepare)(self)
            }
        }
    }
}

pub mod prepare_different_types {
    use super::*;
    to_prepare_trait!(PrepareRef -> Ref <'a, T> for T { where T: ToTokens } {
        type Output = std::iter::Repeat<&'a T>;
        |this| std::iter::repeat(this.0)
    });

    to_prepare_trait!(PrepareRefIntoIter -> RefIntoIter <'a, T> for T { where for<'t> &'t T: IntoIterator, for<'t> <&'t T as IntoIterator>::Item : ToTokens } {
        type Output = <&'a T as IntoIterator>::IntoIter;
        |this| this.0.into_iter()
    });

    to_prepare_trait!(PrepareCloneIntoIter -> CloneIntoIter <'a, T> for &T { where T: Clone + IntoIterator, <T as IntoIterator>::Item : ToTokens } {
        type Output = <T as IntoIterator>::IntoIter;
        |this| this.0.clone().into_iter()
    });
}

#[derive(Clone, Copy)]
pub struct ApplyPrepareQuote;

impl<P: PrepareQuote> FuncMut<P> for ApplyPrepareQuote {
    type Output = P::Output;
    fn call_mut(&mut self, i: P) -> Self::Output {
        i.prepare_quote()
    }
}


pub struct Print;

impl<P: std::fmt::Debug> FuncMut<P> for Print {
    type Output = ();
    fn call_mut(&mut self, i: P) -> Self::Output {
        println!("{i:?}");
    }
}

impl<H, T> ToTokens for Cons<H, T>
where
    H: ToTokens,
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.head.to_tokens(tokens);
        self.tail.to_tokens(tokens);
    }
}

impl ToTokens for End {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // Nothing
    }
}

impl ToTokens for () {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        // Nothing
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InnerRep<R>(R);

impl<R> InnerRep<R> {
    pub fn new(r: R) -> Self {
        Self(r)
    }
}
impl<R> ToTokens for InnerRep<R>
where
    R: ForEach<ApplyPrepareQuote>,
    // <R as krs_hlist::ApplyRef<ApplyPrepareQuote>>::Output: Iterator,
    for<'a> ForEachOut<'a, R, ApplyPrepareQuote>: Iterator,
    for<'a, 't> <ForEachOut<'a, R, ApplyPrepareQuote> as Iterator>::Item: ForEach<ApplyToTokens<'t>>,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token_cons_iter = self.0.for_each(ApplyPrepareQuote);
        for token_cons in token_cons_iter {
            // use krs_hlist::ApplyRef;
            token_cons.for_each(ApplyToTokens(tokens));
        }
    }
}

impl<R> PrepareQuote for &InnerRep<R> {
    type Output = std::iter::Repeat<Self>;
    fn prepare_quote(self) -> Self::Output {
        std::iter::repeat(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InnerRepWithSeparator<R, T>(R, T);

impl<R, T> InnerRepWithSeparator<R, T> {
    pub fn new(r: R, t: T) -> Self {
        Self(r, t)
    }
}

impl<R, T: ToTokens> ToTokens for InnerRepWithSeparator<R, T>
where
    R: ForEach<ApplyPrepareQuote>,
    for<'a> ForEachOut<'a, R, ApplyPrepareQuote>: Iterator,
    for<'a, 't> <ForEachOut<'a, R, ApplyPrepareQuote> as Iterator>::Item: ForEach<ApplyToTokens<'t>>,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut token_cons_iter = self.0.for_each(ApplyPrepareQuote).peekable();
        while let Some(token_cons) = token_cons_iter.next() {
            // use krs_hlist::ApplyRef;
            token_cons.for_each(ApplyToTokens(tokens));
            if token_cons_iter.peek().is_some() {
                self.1.to_tokens(tokens);
            }
        }
    }
}

impl<R, T> PrepareQuote for &InnerRepWithSeparator<R, T> {
    type Output = std::iter::Repeat<Self>;
    fn prepare_quote(self) -> Self::Output {
        std::iter::repeat(self)
    }
}

pub struct PrepareConsWrapper<C>(pub C);

impl<'a, C: ForEach<ApplyPrepareQuote>> PrepareQuote for &'a PrepareConsWrapper<C> {
    type Output = ForEachOut<'a, C, ApplyPrepareQuote>;
    fn prepare_quote(self) -> Self::Output {
        self.0.for_each(ApplyPrepareQuote)
    }
}

#[macro_export]
macro_rules! my_quote {
    ( $($tt:tt)* ) => {{
        use $crate::ForEach;
        use $crate::prepare_different_types::*;
        let mut ts = $crate::TokenStream::new();
        let to_tokens = $crate::End;
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        let mut ti = to_tokens.for_each($crate::ApplyPrepareQuote);
        ti.next().unwrap().for_each($crate::ApplyToTokens(&mut ts));
        ts
    }}
}

#[macro_export]
macro_rules! tokenizer {

    // expand repetition wth separator
    ( {@$sep:tt* $($tt:tt)* } ) => {{
        let to_tokens = $crate::End;
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        match stringify!($sep) {
            "," => $crate::InnerRepWithSeparator::new(to_tokens, $crate::Comma.into()),
            ";" => $crate::InnerRepWithSeparator::new(to_tokens, $crate::SemiColon.into()),
            x => $crate::InnerRepWithSeparator::new(to_tokens, $crate::RawToken(x)),
        }
    }};

    // expand repetition
    ( {@* $($tt:tt)* } ) => {{
        let to_tokens = $crate::End;
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        $crate::InnerRep::new(to_tokens)
    }};

    // expand token
    ( {@$item:ident} ) => {{
        (&$item).as_to_prepare()
    }};

    // extract braces
    ( { $($tt:tt)* } ) => {{
        let to_tokens = $crate::End;
        let to_tokens = to_tokens + $crate::Cons::new($crate::LeftBrace.as_to_prepare());
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        let to_tokens = to_tokens + $crate::Cons::new($crate::RightBrace.as_to_prepare());
        $crate::PrepareConsWrapper(to_tokens)
    }};

    // extract parens
    ( ( $($tt:tt)* ) ) => {{
        let to_tokens = $crate::End;
        let to_tokens = to_tokens + $crate::Cons::new($crate::RawToken("("));
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        let to_tokens = to_tokens + $crate::Cons::new($crate::RawToken(")"));
        $crate::PrepareConsWrapper(to_tokens)
    }};

    // extract bracket
    ( [ $($tt:tt)* ] ) => {{
        let to_tokens = $crate::End;
        let to_tokens = to_tokens + $crate::Cons::new($crate::RawToken("["));
        $( let to_tokens = to_tokens + $crate::Cons::new($crate::tokenizer!($tt)); )*
        let to_tokens = to_tokens + $crate::Cons::new($crate::RawToken("]"));
        $crate::PrepareConsWrapper(to_tokens)
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
        $crate::RawToken(stringify!($tt))
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
}
