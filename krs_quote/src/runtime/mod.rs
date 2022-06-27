use krs_hlist::{ Cons, End, higher_order::prelude::* };

use crate::to_tokens::*;

mod prepare;

pub use prepare::*;

pub struct ApplyToTokens<'t>(pub &'t mut TokenStream);

impl<'t, T: ToTokens> FuncMut<&T> for ApplyToTokens<'t> {
    type Output = ();
    fn call_mut(&mut self, i: &T) {
        i.to_tokens(self.0)
    }
}

pub struct ApplyPrepareQuote;

impl<P: PrepareQuote> FuncMut<P> for ApplyPrepareQuote {
    type Output = P::Output;
    fn call_mut(&mut self, i: P) -> Self::Output {
        i.prepare_quote()
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

