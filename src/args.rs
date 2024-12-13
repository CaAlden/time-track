use clap::Parser;

/// A simple program to track time spans and calculate remaining hours to work
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// How many hours you intend to work (sums with `minutes`)
    /// Usually defaults to 8 hours, but if discount is set then it defaults to 0
    #[arg(long)]
    pub hours: Option<i64>,

    /// How many minutes you intend to work (sums with `hours`)
    #[arg(long, default_value_t = 0)]
    pub minutes: i64,

    /// If true, the the hours and minutes fields are treated as subtracting from 8 hours
    #[arg(long, default_value_t = false)]
    pub discount: bool,
}
