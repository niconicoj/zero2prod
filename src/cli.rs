use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_delimiter = ',')]
    pub profiles: Vec<String>,
}
