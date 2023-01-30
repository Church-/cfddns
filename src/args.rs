use clap::Parser;

#[derive(Clone, Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub config: String,
}
