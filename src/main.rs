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

fn attach(pid: Option<i32>, program: Option<String>, args: Vec<String>) -> Pid {
    match (pid, program) {
        // attach to the already running process
        (Some(pid), None) => {
            if pid <= 0 {
                panic!("invalid pid");
            }
            let pid = Pid::from_raw(pid);
            ptrace::attach(pid.clone()).unwrap();
            pid
        }

        // launch the program then attach to the
        // new child process
        (None, Some(program_path)) => {
            // fork the current process
            match unsafe { fork().unwrap() } {
                ForkResult::Parent { child } => child,
                ForkResult::Child => {
                    // tell the os that the parent is allowed to trace/debug
                    // this child process
                    ptrace::traceme().unwrap();

                    // launch the program
                    let filename = CString::new(program_path.as_str()).unwrap();
                    let mut exec_args = Vec::with_capacity(args.len() + 1);
                    exec_args.push(program_path);
                    exec_args.extend(args);

                    let c_args = exec_args
                        .into_iter()
                        .map(|arg| CString::new(arg).unwrap())
                        .collect::<Vec<_>>();

                    match execvp(&filename, &c_args) {
                        Ok(_) => unreachable!(),
                        Err(err) => panic!("execvp failed: {err}"),
                    }
                }
            }
        }

        // invalid arguments
        _ => {
            panic!("expected --pid <PID> or <PROGRAM>");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let pid = attach(cli.pid, cli.program, cli.args);

    // wait for the child process
    waitpid(pid, None).unwrap();

    // can inspect child prcoess now
}
