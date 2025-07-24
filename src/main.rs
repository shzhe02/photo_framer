use clap::{Parser, ValueEnum};
use framer::{Sizing, frame_image};
use log::error;
use std::{path::PathBuf, process::exit};

mod framer;

#[derive(Clone, ValueEnum, Copy)]
enum OutputType {
    Jpeg,
    Png,
    Webp,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Input folder or image.
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory.
    #[arg(short, long)]
    output: PathBuf,

    /// Aspect ratio to use. Should be provided in the format `<width>:<height>`.
    /// For example: `16:9`, `1:1`, `4.3:2`.
    #[arg(long, alias = "ratio")]
    aspect_ratio: Option<String>,

    /// Output image dimension to use. Should be provided in the format `<width>x<height>`.
    /// For example: `1920x1080`, `1080x1080`, `720x1500`.
    #[arg(long, alias = "dim")]
    dimensions: Option<String>,

    /// Output filetype to use. If not provided, the filetype of the input image will be used.
    #[arg(value_enum, alias = "type")]
    output_filetype: Option<OutputType>,
}

fn main() {
    let accepted_extensions = ["jpeg", "jpg", "png", "webp"];
    env_logger::init();

    let cli = Cli::parse();

    // Validating sizing parameter, making sure only one of either ratio or dimension
    // is provided and that they are formatted correctly.
    let sizing = match (cli.aspect_ratio, cli.dimensions) {
        (None, Some(s)) => {
            let parts = s.split_once('x').unwrap_or_else(|| {
                error!("Output image dimension parameter does not follow expected format.");
                exit(exitcode::CONFIG);
            });
            let width = parts.0.parse::<u32>().unwrap_or_else(|_| {
                error!("Output image dimension width is not a valid integer.");
                exit(exitcode::CONFIG);
            });
            let height = parts.1.parse::<u32>().unwrap_or_else(|_| {
                error!("Output image dimension height is not a valid integer.");
                exit(exitcode::CONFIG);
            });
            Sizing::Dimensions(width, height)
        }
        (Some(s), None) => {
            let parts = s.split_once(':').unwrap_or_else(|| {
                error!("Output image dimension parameter does not follow expected format.");
                exit(exitcode::CONFIG);
            });
            let width = parts.0.parse::<f32>().unwrap_or_else(|_| {
                error!("Output image dimension width is not a valid integer.");
                exit(exitcode::CONFIG);
            });
            let height = parts.1.parse::<f32>().unwrap_or_else(|_| {
                error!("Output image dimension height is not a valid integer.");
                exit(exitcode::CONFIG);
            });
            Sizing::AspectRatio(width, height)
        }
        (Some(_), Some(_)) => {
            error!("Either the aspect ratio or the dimensions can be provided, but not both.");
            exit(exitcode::CONFIG);
        }
        (None, None) => {
            error!("An aspect ratio or output image dimension must be provided.");
            exit(exitcode::CONFIG);
        }
    };
    if !cli.output.exists() || !cli.output.is_dir() {
        error!("The output directory does not exist.");
        exit(exitcode::IOERR);
    }
    if let Ok(dir_files) = cli.input.read_dir() {
        for file in dir_files {
            if file.is_err() {
                continue;
            }
            let file = file.unwrap().path();
            if file.extension().is_some_and(|ext| {
                !accepted_extensions.contains(&ext.display().to_string().as_str())
            }) {
                continue;
            }
            let mut output = cli.output.clone();
            let filename = file.file_name();
            if filename.is_none() {
                continue;
            }
            let filename = filename.unwrap();
            output.push(filename);
            if let Some(filetype) = cli.output_filetype {
                match filetype {
                    OutputType::Jpeg => output.set_extension("jpeg"),
                    OutputType::Png => output.set_extension("png"),
                    OutputType::Webp => output.set_extension("webp"),
                };
            };
            if frame_image(&file, &output, sizing).is_err() {
                error!("Failed to frame image {}", file.display().to_string());
            }
        }
    } else {
        // This assumes the input path leads to a single image.
        let filename = &cli.input.file_name().unwrap_or_else(|| {
            error!("Unable to find input file.");
            exit(exitcode::CONFIG);
        });
        if !accepted_extensions.contains(
            &cli.input
                .extension()
                .unwrap_or_else(|| {
                    error!("Unable to detect input file's filetype.");
                    exit(exitcode::DATAERR);
                })
                .display()
                .to_string()
                .as_str(),
        ) {
            error!(
                "Input file's filetype is unsupported. Use only `jpeg`, `jpg`, `png`, or `webp` files."
            );
            exit(exitcode::CONFIG);
        }
        let mut output = cli.output;
        output.push(filename);
        if let Some(filetype) = cli.output_filetype {
            match filetype {
                OutputType::Jpeg => output.set_extension("jpeg"),
                OutputType::Png => output.set_extension("png"),
                OutputType::Webp => output.set_extension("webp"),
            };
        };
        if frame_image(&cli.input, &output, sizing).is_err() {
            error!("Failed to frame image.");
            exit(exitcode::CANTCREAT);
        }
    }
}
