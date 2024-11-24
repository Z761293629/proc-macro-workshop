#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use derive_builder::Builder;
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env")]
    env: Vec<String>,
    current_dir: Option<String>,
}
struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}
impl CommandBuilder {
    fn executable(&mut self, executable: String) -> &mut Self {
        self.executable = executable;
        self
    }
    fn arg(&mut self, arg: String) -> &mut Self {
        if let Some(ref mut values) = self.args {
            values.push(arg);
        } else {
            self.args = std::option::Option::Some(
                <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([arg])),
            );
        }
        self
    }
    fn args(&mut self, args: Vec<String>) -> &mut Self {
        (/*ERROR*/)
    }
    fn env(&mut self, env: String) -> &mut Self {
        if let Some(ref mut values) = self.env {
            values.push(env);
        } else {
            self.env = std::option::Option::Some(
                <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([env])),
            );
        }
        self
    }
    fn current_dir(&mut self, current_dir: String) -> &mut Self {
        self.current_dir = Some(current_dir);
        self
    }
}
impl CommandBuilder {
    pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
        Ok(Command {
            executable: self
                .executable
                .take()
                .ok_or(
                    ::alloc::__export::must_use({
                        let res = ::alloc::fmt::format(
                            format_args!("{0} is not set", "executable"),
                        );
                        res
                    }),
                )?,
            args: self.args.take().unwrap_or(Vec::new()),
            env: self.env.take().unwrap_or(Vec::new()),
            current_dir: self.current_dir.take(),
        })
    }
}
impl Command {
    fn builder() -> CommandBuilder {
        CommandBuilder {
            executable: None,
            args: None,
            env: None,
            current_dir: None,
        }
    }
}
fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();
    match (&command.executable, &"cargo") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    match (
        &command.args,
        &<[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new(["build", "--release"])),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
