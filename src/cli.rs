use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "speedtest-tui",
    version,
    about = "⚡ Professional internet speed test in your terminal",
    long_about = None
)]
pub struct Cli {
    /// Use a specific test server by ID
    #[arg(short = 's', long = "server", value_name = "ID")]
    pub server: Option<String>,

    /// Skip upload phase
    #[arg(short = 'n', long = "no-upload")]
    pub no_upload: bool,

    /// Run silently, print JSON result on exit
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Export results: json | csv | png
    #[arg(short = 'e', long = "export", value_name = "FORMAT")]
    pub export: Option<ExportFormat>,

    /// Force theme: dark | light
    #[arg(short = 't', long = "theme", value_name = "THEME")]
    pub theme: Option<ThemeArg>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Png,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ThemeArg {
    Dark,
    Light,
}
