use krs_hlist_pm::hlist_ty;

type MyList<'a> = hlist_ty!(u32, i8, &'a str);

fn main() {

    let list = MyList::default();

    println!("{list:?}");

}