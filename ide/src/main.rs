use std::path::PathBuf;
use structopt::StructOpt;

mod ssa;
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
