use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "crowd-simulator")]
#[command(about = "VECTOR Conference Crowd & Telemetry Load Simulator", long_about = None)]
pub struct CliArgs {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    pub target: String,

    #[arg(short, long, default_value = "wave")]
    pub mode: String,

    #[arg(short, long, default_value_t = 25)]
    pub concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = CliArgs::parse();
    println!(
        "Crowd Simulator initialized [target={}, mode={}, concurrency={}]",
        args.target, args.mode, args.concurrency
    );
    Ok(())
}
