use krs_quote::krs_quote;

fn main() {
    let t = "there";

    let v = [1u8, 2, 3];

    let q = krs_quote!( hey {@,* {@v} {@t} } );

    println!("{q}");
}
