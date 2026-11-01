use std::process::{Command, ExitStatus};
use std::io;

#[inline(always)]
fn run_command(cmd: &str) -> Result<ExitStatus, io::Error> {
    let argv = cmd.split(' ').collect::<Vec<&str>>();
    let prog = argv[0];
    let prog_args = &argv[1..];
    Ok(Command::new(prog).args(prog_args).spawn()?.wait()?)
}

fn main() -> Result<(), io::Error> {
    println!("Logging in...");
    run_command("aws login")?;
    todo!();
    Ok(())
}