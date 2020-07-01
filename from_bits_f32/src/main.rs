fn main() {
    let args = ::std::env::args().collect::<Vec<_>>();

    let v = run(args
        .get(1)
        .expect("not enough arguments")
        .parse()
        .expect("number not u64"));

    println!("{}", v);
}

fn run(s: u32) -> f32 {
    let v = f32::from_bits(s);

    v
}
