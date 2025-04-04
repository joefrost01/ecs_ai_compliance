use clap::Parser;

// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Number of AI events to process per second
    #[arg(short, long, default_value_t = 100000)]
    pub rate: u32,

    /// Reporting interval in seconds
    #[arg(short, long, default_value_t = 5)]
    pub interval: u64,

    /// Number of worker threads (defaults to number of logical cores)
    #[arg(short, long)]
    pub threads: Option<usize>,
}

// Components - kept small for performance
pub struct AIService {
    pub name_idx: u8,  // Index into a static array rather than storing strings
    pub vendor_idx: u8,
}

pub struct Usage {
    pub department_idx: u8,
    pub data_sensitivity: u8, // 0-100 scale
}

pub struct ComplianceStatus {
    pub flags: u8, // Bit flags for different compliance statuses
}

pub struct RiskAssessment {
    pub score: u8, // 0-100 scale
    pub factor_flags: u16, // Bit flags for different risk factors
}