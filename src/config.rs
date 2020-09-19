use std::time::Duration;

use once_cell::sync::OnceCell;
use structopt::StructOpt;

static CONFIG: OnceCell<Config> = OnceCell::new();

/// Return a `Config` built from command line args
pub fn global() -> &'static Config {
    CONFIG.get_or_init(Config::from_args)
}

#[derive(Clone, Debug, StructOpt)]
pub struct Config {
    #[structopt(short, long, default_value = "1024", display_order = 0)]
    /// Width of render
    pub width: usize,

    #[structopt(short, long, default_value = "576", display_order = 1)]
    /// Height of render
    pub height: usize,

    // Run at 30 fps
    #[structopt(skip = Duration::from_secs_f64(1. / 30.))]
    /// Controls the framerate
    pub delay: Duration,

    #[structopt(
        help = "Disable antialiasing",
        short = "n",
        long = "no-aa",
        // Disable antialiasing if the flag is given at least once
        parse(from_occurrences = invert_bool),
    )]
    /// Controls antialiasing
    pub antialias: bool,

    #[structopt(short, long = "samples", default_value = "100")]
    /// Number of samples per pixel
    pub samples_per_pixel: u16,

    #[structopt(short, long = "max-depth", default_value = "100")]
    /// Maximum ray bounce depth
    pub max_ray_depth: u32,

    #[structopt(short = "r", long = "rng")]
    /// Use a specific seed for the rng.
    pub seed: Option<u64>,
}

fn invert_bool(i: u64) -> bool {
    i == 0
}
