mod mosh;

use std::{
    error::Error,
    fmt::Display,
    fs,
    process::{Child, Command, Stdio},
};

use clap::Parser;

#[derive(clap::Parser)]
struct Args {
    input: String,
    output: String,

    /// If specified, the video will not be remuxed.
    /// This is recommended to be specified when the input is a temporary file generated by this program.
    #[arg(short, long)]
    premuxed: bool,

    /// If specified, removes the intermediate file.
    /// Not specified by default to allow for reuse of the temporary.
    #[arg(short, long)]
    remove_temporary: bool,

    /// A value from 0 to 1 dictating where in the video the shuffle process starts.
    #[arg(short, long, default_value("0.0"))]
    shuffle_start_fraction: f32,
}

fn remux_all_pframes(
    input_path: impl Display,
    output_path: impl Display,
) -> Result<Child, Box<dyn Error>> {
    Ok(Command::new("ffmpeg")
        .args([
            "-i",
            &input_path.to_string(),
            "-y",
            "-c:v",
            "libx264",
            "-x264-params",
            "keyint=99999999:bframes=0",
            "-f",
            "mp4",
            &output_path.to_string(),
        ])
        // .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let temporary = if args.premuxed {
        args.input.clone()
    } else {
        args.input.clone() + ".mux.tmp"
    };

    if !args.premuxed {
        println!("converting all possible frames to p-frames...");
        if !remux_all_pframes(&args.input, &temporary)?
            .wait()?
            .success()
        {
            panic!("internal remux error");
        }
    }
    println!("datamoshing video...");
    let _ = fs::remove_file(&args.output);
    mosh::mosh(&temporary, &args.output, args.shuffle_start_fraction.clamp(0.0, 1.0))?;
    if args.remove_temporary && !args.premuxed {
        println!("removing temporaries...");
        fs::remove_file(temporary)?;
    }
    println!("video has been datamoshed!");

    Ok(())
}
