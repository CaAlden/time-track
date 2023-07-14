use clap::ValueEnum;

#[derive(ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Modes {
    /// Perform calculations by treating stdin as a stream of time pairs separated by newlines
    TimeTable,
    /// Perform live tracking of the user's time until an EOF character is received.
    Live,
    /// Predict a user's day end time (assuming a 9 hour work day) based on the user's start time
    /// and the duration of a user's break.
    Prediction,
}

impl Modes {
    /// Determine whether or not the variant supports piped input vs being used as a CLI tool
    /// directly by a human
    pub fn supports_piped_input(&self) -> bool {
        match self {
            Modes::TimeTable => true,
            Modes::Live => false,
            Modes::Prediction => false,
        }
    }
}


impl Default for Modes {
    fn default() -> Self {
        Self::TimeTable
    }
}

impl std::fmt::Display for Modes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}
