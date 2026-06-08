use clap::Parser;
use nix::{
    sys::{ptrace, wait::waitpid},
    unistd::{ForkResult, Pid, execvp, fork},
};
use std::ffi::CString;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    pid: Option<i32>,

    program: Option<String>,

    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(pid) = cli.pid {
        // parse the pid
        if pid <= 0 {
            panic!("invalid pid");
        }
        ptrace::attach(Pid::from_raw(pid)).unwrap();
    } else if let Some(program) = cli.program {
        // parse the program name
        match unsafe { fork().unwrap() } {
            ForkResult::Parent { child } => {
                waitpid(child, None).unwrap();

                // child is paused and we can inspect anything

                todo!()
            }
            ForkResult::Child => {
                // we need to exec the file path
                // and call traceme
                ptrace::traceme().unwrap();

                let filename = CString::new(program.as_str()).unwrap();
                let mut exec_args = Vec::with_capacity(cli.args.len() + 1);
                exec_args.push(program);
                exec_args.extend(cli.args);

                let c_args = exec_args
                    .iter()
                    .map(|arg| CString::new(arg.as_str()).unwrap())
                    .collect::<Vec<_>>();

                match execvp(&filename, &c_args) {
                    Ok(_) => unreachable!(),
                    Err(err) => panic!("execvp failed: {err}"),
                }
            }
        }
    } else {
        panic!("expected --pid <PID> or <PROGRAM>");
    }
}
