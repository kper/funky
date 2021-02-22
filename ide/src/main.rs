use std::path::PathBuf;
use structopt::StructOpt;
#[macro_use] extern crate lalrpop_util;

mod ssa;

lalrpop_mod!(pub grammar);

#[cfg(test)]
mod tests;

#[derive(Debug, StructOpt)]
#[structopt(name = "ide", about = "IDE framework solver")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}


fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
