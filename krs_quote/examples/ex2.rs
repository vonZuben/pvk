use krs_quote::my_quote;

fn main() {

    let t = "there";

    let v = [1u8, 2, 3];

    let q = my_quote!( hey {@,* {@v} {@t} } );

    struct Print;

    impl<T: std::fmt::Debug> krs_quote::FuncMut<T> for Print {
        type Output = ();

        fn call_mut(&mut self, i: T) -> Self::Output {
            println!("{i:?}");
        }
    }

    println!("{q}");
}
