use krs_quote::krs_quote;
use krs_quote::Token;

fn main() {
    let name: Token = "A".into();

    let members = ["a", "b", "c"].map(Token::from);
    let values = [1, 2, 3];

    let o1 = Some(4);
    let o2: Option<i8> = Some(1);

    let os = [o1, o2];

    let o3 = Some(1);

    let code = krs_quote! {
        struct {@name} {
            {@,* {@members} : i32 }
        }
        impl {@name} {
            fn new() -> Self {
                todo!()
            }
            fn tst() {
                let v = [ {@* {@members},} ];
                fn h<{@name}>(_: _){}
                f({@name})
            }
        }
        macro_rules! a {
            () => {
                {@*
                    let a = {@os};
                }
            }
        }
        fn main() {
            let s = {@name} {
                {@,* {@members} : {@values} }
            };
            {@o3}
        }
    };

    println!("{code}");
}
