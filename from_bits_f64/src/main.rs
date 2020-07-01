fn main() {
    let args = ::std::env::args().collect::<Vec<_>>();

    let v = run(args
        .get(1)
        .expect("not enough arguments")
        .parse()
        .expect("number not u64"));

    println!("{}", v);
}

fn run(s: u64) -> f64 {
    let v = f64::from_bits(s);

    v
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_f64_parse() {
        assert_eq!(run(0x4029000000000000), 12.5);
    }

    #[test]
    fn test_f64_parse_2() {
        assert_eq!(run("4607182418800017408".parse().unwrap()), 1.0);
    }
}
