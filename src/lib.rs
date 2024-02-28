//! Wraps the ffpmeg cli, using `-progress` to report progress
//!
//! Sometimes you just want a simple way to use ffmpeg. Most crates just use ffi, leading to
//! complicated interfaces. `ffmpeg_cli` avoids this by wrapping the cli, for when you don't need
//! the flexibility the real ffmpeg api gives you.
//!
//! ```no_run
//! use std::process::Stdio;
//!
//! use ffmpeg_cli::{FfmpegBuilder, File, Parameter};
//! use futures::{future::ready, StreamExt};
//!
//! #[tokio::main]
//! async fn main() {
//!     let builder = FfmpegBuilder::new()
//!         .stderr(Stdio::piped())
//!         .option(Parameter::Single("nostdin"))
//!         .option(Parameter::Single("y"))
//!         .input(File::new("input.mkv"))
//!         .output(
//!             File::new("output.mp4")
//!                 .option(Parameter::KeyValue("vcodec", "libx265"))
//!                 .option(Parameter::KeyValue("crf", "28")),
//!         );
//!
//!     let ffmpeg = builder.run().await.unwrap();
//!
//!     ffmpeg
//!         .progress
//!         .for_each(|x| {
//!             dbg!(x.unwrap());
//!             ready(())
//!         })
//!         .await;
//!
//!     let output = ffmpeg.process.wait_with_output().unwrap();
//!
//!     println!(
//!         "{}\nstderr:\n{}",
//!         output.status,
//!         std::str::from_utf8(&output.stderr).unwrap()
//!     );
//! }
//! ```
#![warn(missing_docs)]

use std::process::{Command, Stdio};

mod runner;

#[doc(inline)]
pub use runner::*;

/// The main struct which is used to set up ffmpeg.
#[derive(Debug)]
pub struct FfmpegBuilder<'a> {
    /// The global options.
    pub options: Vec<Parameter<'a>>,
    /// The input files.
    pub inputs: Vec<File<'a>>,
    /// The output files.
    pub outputs: Vec<File<'a>>,

    /// The command that's run for ffmpeg. Usually just `ffmpeg`
    pub ffmpeg_command: &'a str,
    /// Passed as [Command::stdin]
    pub stdin: Stdio,
    /// Passed as [Command::stdout]
    pub stdout: Stdio,
    /// Passed as [Command::stderr]
    pub stderr: Stdio,
}

/// A file that ffmpeg operates on.
///
/// This can be an input or output, it depends on what you add it as.
#[derive(Debug)]
pub struct File<'a> {
    /// The url of the file.
    ///
    /// As with ffmpeg, just a normal path works.
    pub url: &'a str,
    /// The options corresponding to this file.
    pub options: Vec<Parameter<'a>>,
}

/// A global or file option to be passed to ffmpeg.
#[derive(Debug, Clone, Copy)]
pub enum Parameter<'a> {
    /// An option which does not take a value, ex. `-autorotate`.
    ///
    /// `-autorotate` would be represented as `Single("autorotate")`,
    /// as the `-` is inserted automatically.
    Single(&'a str),
    /// An option that takes a key and a value, ex. `-t 10`.
    ///
    /// `-t 10` would be represented as `KeyValue("t", "10")`, as
    /// the `-` is inserted automatically.
    KeyValue(&'a str, &'a str),
}

impl<'a> FfmpegBuilder<'a> {
    /// Gets a [FfmpegBuilder] with nothing set
    pub fn new() -> FfmpegBuilder<'a> {
        FfmpegBuilder {
            options: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            ffmpeg_command: "ffmpeg",
            stdin: Stdio::null(),
            stdout: Stdio::null(),
            stderr: Stdio::null(),
        }
    }

    /// Adds an option.
    pub fn option(mut self, option: Parameter<'a>) -> Self {
        self.options.push(option);

        self
    }

    /// Adds an input.
    pub fn input(mut self, input: File<'a>) -> Self {
        self.inputs.push(input);

        self
    }

    /// Adds an output.
    pub fn output(mut self, output: File<'a>) -> Self {
        self.outputs.push(output);

        self
    }

    /// Sets stdin.
    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = stdin;

        self
    }

    /// Sets stdout.
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = stdout;

        self
    }

    /// Sets stderr.
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = stderr;

        self
    }

    /// Turns it into a command, consuming the builder.
    ///
    /// This has to consume the builder for stdin, etc to work
    /// Note that usually you want to use [`Self::run()`], not call this directly
    pub fn to_command(self) -> Command {
        let mut command = Command::new(self.ffmpeg_command);

        for option in self.options {
            option.push_to(&mut command);
        }
        for input in self.inputs {
            input.push_to(&mut command, true);
        }
        for output in self.outputs {
            output.push_to(&mut command, false)
        }

        command.stdin(self.stdin);
        command.stdout(self.stdout);
        command.stderr(self.stderr);

        command
    }
}

impl<'a> File<'a> {
    /// Gets a file without any options set.
    pub fn new(url: &'a str) -> File {
        File {
            url,
            options: Vec::new(),
        }
    }

    /// Adds an option.
    pub fn option(mut self, option: Parameter<'a>) -> Self {
        self.options.push(option);

        self
    }

    fn push_to(&self, command: &mut Command, input: bool) {
        for option in &self.options {
            option.push_to(command);
        }

        if input {
            command.arg("-i");
        }
        command.arg(&self.url);
    }
}

impl<'a> Parameter<'a> {
    fn push_to(&self, command: &mut Command) {
        match &self {
            Parameter::Single(arg) if arg.len() > 0 => command.arg("-".to_owned() + arg),
            Parameter::KeyValue(key, value) => {
                command.arg("-".to_owned() + key);
                command.arg(value)
            }
            _ => command.arg(""),
        };
    }
}
