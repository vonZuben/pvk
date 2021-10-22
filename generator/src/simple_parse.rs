#[derive(Clone)]
pub struct TokenIter<'a> {
    s: &'a str,
    cursor: usize,
}

impl<'a> TokenIter<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        // base case, empty s
        if self.cursor >= self.s.len() {
            return None;
        }
        else {
            const SPECIAL_CHARS: &[char] = &['[', ']', '{', '}', '(', ')', ':', ';', ',', '*', '&', '|', '=', '\'', '\"'];
        
            // find start of next token
            let mut start = self.cursor;
            let mut end = 0;
            let chars = self.s[start..].chars();
            
            // loop until first non whitespace/special char or until end of chars
            for c in chars {
                // skip whitespace
                if c == ' ' {
                    start += 1;
                    continue;
                }
                // immidiate return for special chars
                else if SPECIAL_CHARS.contains(&c) {
                    // want to just return single char, but need to return from slice so type matches
                    end = start + c.len_utf8();
                    self.cursor = end;
                    return Some(&self.s[start..end]);
                }
                // found first none whitespace/special char
                else {
                    end = start + c.len_utf8(); // initialize end to be 1 char after start
                    break;
                }
            }
            
            // check if start hit end of s, e.g. if the remaining chars were all white space
            if start >= self.s.len() {
                return None;
            }
            else { 
                // find end of next token
                // let mut end = start + s[];
                let chars = self.s[end..].chars();
                for c in chars {
                    // found boundary for end of next token
                    if c == ' ' || SPECIAL_CHARS.contains(&c) {
                        break;
                    }
                    else {
                        end += c.len_utf8();
                    }
                }
                
                // either we found a boundary, or we got to the end of s
                self.cursor = end;
                Some(&self.s[start..end])
            }
        }
    }
}

use std::fmt;

type MyErr<'a> = ();

type Res<I, O> = Result<(I, O), MyErr<'static>>;

pub trait ParseFn<I, O> : FnMut(I) -> Res<I, O> {}
impl<F, I, O>  ParseFn<I, O> for F where F: FnMut(I) -> Res<I, O> {}

pub trait Parse<I, O> {
    fn parse(&mut self, input: I) -> Res<I, O>;
}

impl<I, O, F> Parse<I, O> for F
where
    F: FnMut(I) -> Res<I, O>,
{
    fn parse(&mut self, input: I) -> Res<I, O> {
        self(input)
    }
}

pub fn token<'a, O, I: Iterator<Item = O>>() -> impl ParseFn<I, O> {
    move |mut input: I| input.next().and_then(|o| Some((input, o))).ok_or(())
}

pub fn tag<'a, T: Eq, I: Iterator<Item = T> + 'a>(tag: T) -> impl ParseFn<I, T> {
    move |mut input: I| {
        input
            .next()
            .and_then(|token| {
                if token == tag {
                    Some((input, token))
                } else {
                    None
                }
            })
            .ok_or(())
    }
}

pub fn followed<I, O1, O2>(
    mut p1: impl Parse<I, O1>,
    mut p2: impl Parse<I, O2>,
) -> impl ParseFn<I, (O1, O2)> {
    move |input: I| {
        let (input, o1) = p1.parse(input)?;
        let (input, o2) = p2.parse(input)?;
        Ok((input, (o1, o2)))
    }
}

pub fn delimited<I, O1, O2, O3>(
    mut p1: impl Parse<I, O1>,
    mut p2: impl Parse<I, O2>,
    mut p3: impl Parse<I, O3>,
) -> impl ParseFn<I, (O1, O2, O3)> {
    move |input: I| {
        let (input, o1) = p1.parse(input)?;
        let (input, o2) = p2.parse(input)?;
        let (input, o3) = p3.parse(input)?;
        Ok((input, (o1, o2, o3)))
    }
}

pub fn opt<I: Clone, O>(mut p: impl Parse<I, O>) -> impl ParseFn<I, Option<O>> {
    move |input: I| {
        let i_old = input.clone();
        p.parse(input)
            .map(|(i, o)| (i, Some(o)))
            .or(Ok((i_old, None)))
    }
}

pub fn repeat<I: Clone, O>(mut input: I, mut p: impl Parse<I, O>, mut f: impl FnMut(O)) -> Res<I, ()> {
    loop {
        let oldi = input.clone();
        match p.parse(input) {
            Ok((rest, o)) => {
                input = rest;
                f(o);
            }
            Err(_) => {
                return Ok((oldi, ()))
            }
        }
    }
}

#[cfg(test)]
mod test_token_iter {
    use super::*;
    #[test]
    fn parse_different_inputs() {
        let s = "hey there (buddy)=            [tom]";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "    my name = \"john\"";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "hello";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "6";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "::";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "                                        ";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "(                  !                      )";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
        
        let s = "Grüße, Jürgen ❤";
        let i = TokenIter::new(s);
        for t in i {
            println!("{}", t);
        }
    }
}