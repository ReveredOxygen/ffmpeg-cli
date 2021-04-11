use std::process::Stdio;

use ffmpeg_cli::{FfmpegBuilder, File, Parameter};
use futures::{future::ready, StreamExt};

#[tokio::main]
async fn main() {
    let builder = FfmpegBuilder::new()
        .stderr(Stdio::piped())
        .option(Parameter::Single("nostdin"))
        .option(Parameter::Single("y"))
        .input(File::new("input.mkv"))
        .output(
            File::new("output.mp4")
                .option(Parameter::KeyValue("vcodec", "libx265"))
                .option(Parameter::KeyValue("crf", "28")),
        );

    let ffmpeg = builder.run().await.unwrap();

    ffmpeg
        .progress
        .for_each(|x| {
            dbg!(x.unwrap());
            ready(())
        })
        .await;

    let output = ffmpeg.process.wait_with_output().unwrap();

    println!(
        "{}\nstderr:\n{}",
        output.status,
        std::str::from_utf8(&output.stderr).unwrap()
    );
}
