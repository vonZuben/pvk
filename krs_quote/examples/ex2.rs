use krs_quote::my_quote;

fn main() {

    let t = "there";

    let v = [1u8, 2, 3];

    let q = my_quote!( hey {@,* {@v} {@t} } );

    println!("{q}");
}
