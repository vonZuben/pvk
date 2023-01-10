extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, Span, Delimiter, Ident, Group, Punct, Spacing, Literal};

use std::iter;

#[proc_macro]
pub fn hlist(input: TokenStream) -> TokenStream {
	let mut tt_iter = input.into_iter().peekable();
	let output = make_node(&mut tt_iter);
	if let Some(extra_tt) = tt_iter.next() {
		return compile_error("unexpected token", extra_tt.span());
	}
	else {
		output
	}
}

fn make_node<I: Iterator<Item=TokenTree>>(tt_iter: &mut iter::Peekable<I>) -> TokenStream {
	let next = tt_iter.peek();

	//dbg!(next);
	if next.is_none() || matches!(next, Some(TokenTree::Punct(x)) if x.as_char() == ',') {
		make_path(["krs_hlist", "End"])
	}
	else {
		let head: TokenStream = UntilComma::new(tt_iter).collect();
		let tail: TokenStream = make_node(tt_iter);
		let group: TokenStream = [
			head,
			TokenTree::Punct(Punct::new(',', Spacing::Alone)).into(),
			tail,
		].into_iter().collect();

		[
			make_path(["krs_hlist", "Cons", "new"]),
			TokenTree::Group(Group::new(Delimiter::Parenthesis, group)).into(),
		].into_iter().collect()
	}
}

#[proc_macro]
pub fn hlist_ty(input: TokenStream) -> TokenStream {
	let mut tt_iter = input.into_iter().peekable();
	let output = make_node_ty(&mut tt_iter);
	if let Some(extra_tt) = tt_iter.next() {
		return compile_error("unexpected token", extra_tt.span());
	}
	else {
		output
	}
}

fn make_node_ty<I: Iterator<Item=TokenTree>>(tt_iter: &mut iter::Peekable<I>) -> TokenStream {
	let next = tt_iter.peek();

	//dbg!(next);
	if next.is_none() || matches!(next, Some(TokenTree::Punct(x)) if x.as_char() == ',') {
		make_path(["krs_hlist", "End"])
	}
	else {
		let head: TokenStream = UntilComma::new(tt_iter).collect();
		let tail: TokenStream = make_node_ty(tt_iter);

		[
			make_path(["krs_hlist", "Cons"]),
			TokenTree::Punct(Punct::new('<', Spacing::Alone)).into(),
			head,
			TokenTree::Punct(Punct::new(',', Spacing::Alone)).into(),
			tail,
			TokenTree::Punct(Punct::new('>', Spacing::Alone)).into(),
		].into_iter().collect()
	}
}

#[cfg(not(feature = "re-export"))]
fn make_path<'a>(names: impl IntoIterator<Item=&'a str>) -> TokenStream {
	let level = |name| {
		[
			TokenTree::Punct(Punct::new(':', Spacing::Joint)),
			TokenTree::Punct(Punct::new(':', Spacing::Alone)),
			TokenTree::Ident(Ident::new(name, Span::call_site())),
		].into_iter()
	};

	names.into_iter().map(level).flatten().collect()
}

#[cfg(feature = "re-export")]
fn make_path<'a>(names: impl IntoIterator<Item=&'a str>) -> TokenStream {
	let not_last = |name| -> TokenStream {
		[
			TokenTree::Ident(Ident::new(name, Span::call_site())),
			TokenTree::Punct(Punct::new(':', Spacing::Joint)),
			TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		].into_iter().collect()
	};

	let last = |name| {
		[
			TokenTree::Ident(Ident::new(name, Span::call_site())),
		].into_iter().collect()
	};

	let mut names = names.into_iter().peekable();

	let mapper = std::iter::from_fn(|| {
		let name = names.next()?;
		if names.peek().is_none() {
			Some(last(name))
		}
		else {
			Some(not_last(name))
		}
	});

	mapper.collect()
}

fn compile_error(msg: &str, with_span: Span) -> TokenStream {
	let error = TokenTree::Literal(Literal::string(msg));
	[
		TokenTree::Ident(Ident::new("compile_error", with_span)),
		TokenTree::Punct(Punct::new('!', Spacing::Alone)),
		TokenTree::Group(Group::new(Delimiter::Parenthesis, error.into())),
	].into_iter().collect()
}

// repeatedly call next on the iterator until a comma
// can potentially resume after a comma
struct UntilComma<'a, I>{
	iter: &'a mut I,
}

impl<'a, I> UntilComma<'a, I> {
	fn new(i: &'a mut I) -> Self {
		Self{
			iter: i,
		}
	}
}

impl<I> Iterator for UntilComma<'_, I>
where
	I: Iterator<Item=TokenTree>,
{
	type Item = I::Item;
	fn next(&mut self) -> Option<Self::Item> {
		let item = self.iter.next()?;

		if matches!(item, TokenTree::Punct(ref x) if x.as_char() == ',') {
			None
		}
		else {
			Some(item)
		}
	}
}