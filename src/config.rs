use once_cell::sync::OnceCell;
use std::time::Duration;
use structopt::StructOpt;

static CONFIG: OnceCell<Config> = OnceCell::new();

/// Return a `Config` built from command line args
pub fn global() -> &'static Config {
    CONFIG.get_or_init(Config::from_args)
}

#[derive(Clone, Debug, StructOpt)]
pub struct Config {
    /// Width of render
    #[structopt(short, long, default_value = "1024", display_order = 0)]
    pub width: usize,

    /// Height of render
    #[structopt(short, long, default_value = "576", display_order = 1)]
    pub height: usize,

    // Run at 30 fps
    #[structopt(skip = Duration::from_secs_f64(1. / 30.))]
    pub delay: Duration,

    // Control antialiasing
    #[structopt(
        help = "Disable antialiasing",
        short = "n",
        long = "no-aa",
        // Disable antialiasing if the flag is given at least once
        parse(from_occurrences = invert_bool),
    )]
    pub antialias: bool,

    /// Number of samples per pixel
    #[structopt(short, long = "samples", default_value = "100")]
    pub samples_per_pixel: u16,

    /// Maximum ray bounce depth
    #[structopt(short, long = "max-depth", default_value = "100")]
    pub max_ray_depth: u32,

    /// Seed the rng. Otherwise, use OS entropy.
    #[structopt(short = "r", long = "rng")]
    pub seed: Option<u64>,
}

fn invert_bool(i: u64) -> bool { i == 0 }
