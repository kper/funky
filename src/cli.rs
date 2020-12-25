use regex::Regex;
use regex::RegexSet;
use crate::value::Value;
use crate::value::Value::*;

pub fn parse_args(args: Vec<String>) -> Vec<Value> {
    let matchers = &[
        r"I32\((-?[0-9]+)\)",
        r"I64\((-?[0-9]+)\)",
        r"F32\((-?[0-9]+(\.[0-9]+)?)\)",
        r"F64\((-?[0-9]+(\.[0-9]+)?)\)",
        r"F32\(inf\)",
        r"F32\(-inf\)",
        r"F64\(inf\)",
        r"F64\(-inf\)",
        r"F32\(nan\)",
        r"F64\(nan\)",
    ];
    let set = RegexSet::new(matchers).unwrap();
    args.iter()
        .map(|a| {
            let matches = set.matches(a);
            debug!("matches: {:?}", matches);
            if matches.matched(0) {
                let re = Regex::new(matchers[0]).unwrap();
                let caps = re.captures(a).unwrap();
                match caps[1].parse::<i32>() {
                    Ok(x) => I32(x),
                    _ => I32(caps[1].parse::<u32>().unwrap() as i32),
                }
            } else if matches.matched(1) {
                let re = Regex::new(matchers[1]).unwrap();
                let caps = re.captures(a).unwrap();
                match caps[1].parse::<i64>() {
                    Ok(x) => I64(x),
                    _ => I64(caps[1].parse::<u64>().unwrap() as i64),
                }
            } else if matches.matched(2) {
                let re = Regex::new(matchers[2]).unwrap();
                let caps = re.captures(a).unwrap();
                F32(caps[1].parse::<f32>().unwrap())
            } else if matches.matched(3) {
                let re = Regex::new(matchers[3]).unwrap();
                let caps = re.captures(a).unwrap();
                F64(caps[1].parse::<f64>().unwrap())
            } else if matches.matched(4) {
                F32(f32::INFINITY)
            } else if matches.matched(5) {
                F32(-f32::INFINITY)
            } else if matches.matched(6) {
                F64(f64::INFINITY)
            } else if matches.matched(7) {
                F64(-f64::INFINITY)
            } else if matches.matched(8) {
                F32(f32::NAN)
            } else if matches.matched(9) {
                F64(f64::NAN)
            } else {
                panic!("Invalid parameter type specified {}", a);
            }
        })
        .collect()
}
