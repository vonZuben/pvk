

// trace_macros!(true);

use krs_quote::my_quote;

fn main() {

    let t = "there";

    let v = [1u8, 2, 3];

    let q = my_quote!( hey {@,* {@v}} );

    // let q = {
    //     use krs_quote::ApplyRef;
    //     use krs_quote::prepare_different_types::*;
    //     use krs_quote::PrepareQuote;
    //     let mut ts = krs_quote::TokenStream::new();
    //     let to_tokens = krs_quote::End;
    //     // let to_tokens = to_tokens + krs_quote::Cons::new(krs_quote::RawToken("hey"));
    //     let to_tokens = to_tokens + krs_quote::Cons::new(t.as_to_prepare());
    //     // let to_tokens = to_tokens + krs_quote::Cons::new(krs_quote::InnerRepWithSeparator::new({
    //     //     // let mut ts = krs_quote::TokenStream::new();
    //     //     let to_tokens2 = krs_quote::End;
    //     //     let to_tokens2 = to_tokens2 + krs_quote::Cons::new(v.as_to_prepare());
    //     //     to_tokens2
    //     // }, krs_quote::RawToken(",")));
    //     let to_tokens = to_tokens + 
    //         krs_quote::Cons::new(krs_quote::InnerRep::new(krs_quote::Cons::new(v.as_to_prepare())));
    //     // let to_tokens = to_tokens + krs_quote::Cons::new(v.as_to_prepare());
    //     let mut ti = to_tokens.apply_ref(krs_quote::ApplyPrepareQuote);
    //     // ti.next().unwrap().apply_ref($crate::TmpOp);
    //     let tmp = ti.next().unwrap();
    //     tmp.apply_ref(krs_quote::ApplyToTokens(&mut ts));
    //     ts
    //     // println!("{ts}");
    // };

    struct Print;

    impl<T: std::fmt::Debug> krs_quote::FuncMut<T> for Print {
        type Output = ();

        fn call_mut(&mut self, i: T) -> Self::Output {
            println!("{i:?}");
        }
    }

    // q.apply_ref(Print);
    println!("{q}");
}