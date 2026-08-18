use clap::Parser;
use lazy_static::lazy_static;
use serde::Serialize;
use std::net::IpAddr;

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
}

#[derive(Parser, Debug, Clone, Serialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env = "PORT", default_value_t = 28080)]
    pub port: u16,

    #[arg(short, long, env = "BIND", default_value_t = IpAddr::from([0, 0, 0, 0]))]
    pub bind: IpAddr,

    #[arg(long, env = "PUBLIC_DIR", default_value = "public")]
    pub public_dir: String,

    #[arg(long, env = "VIEWS_DIR", default_value = "views")]
    pub views_dir: String,

    #[arg(long, env = "MAX_UPLOAD_SIZE_MB", default_value_t = 100)]
    pub max_upload_size_mb: usize,
}

