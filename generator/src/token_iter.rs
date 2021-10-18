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

#[cfg(test)]
mod test {
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