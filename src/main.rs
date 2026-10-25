// No console behind the window in the distributed version
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    lotpix::cli::run(lotpix::cli::parse(std::env::args_os().skip(1)))
}
