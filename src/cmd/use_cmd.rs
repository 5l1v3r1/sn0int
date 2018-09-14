use errors::*;

use shell::Readline;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
pub struct Args {
    module: String,
}

pub fn run(rl: &mut Readline, args: &[String]) -> Result<()> {
    let args = Args::from_iter_safe(args)?;

    let module = rl.engine().get(&args.module)?.clone();
    rl.set_module(module);

    Ok(())
}
