use clap::Parser;

/// A simple program to track time spans and calculate remaining hours to work
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// How many hours you intend to work (sums with `minutes`)
    #[arg(long, default_value_t = 8)]
    pub hours: i64,

    /// How many minutes you intend to work (sums with `hours`)
    #[arg(long, default_value_t = 0)]
    pub minutes: i64,
}
