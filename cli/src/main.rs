use std::path::PathBuf;
use structopt::StructOpt;

/// GDLK command-line compiler.
#[derive(StructOpt, Debug)]
#[structopt(name = "gdlkc")]
enum Opt {
    /// Compiles source code
    #[structopt(name = "compile")]
    Compile {
        #[structopt()]
        input: PathBuf,
    },
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}
