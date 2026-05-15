use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let message = "\
fixture3 ships prebuilt binaries. Install it with:
  cargo binstall fixture3

If cargo-binstall is missing:
  cargo install cargo-binstall
";

    match io::stderr().write_all(message.as_bytes()) {
        Ok(()) => ExitCode::from(1),
        Err(_) => ExitCode::from(2),
    }
}
