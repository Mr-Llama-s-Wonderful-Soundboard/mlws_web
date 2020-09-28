use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
	/// IPs to bind to
	#[structopt(short, long, default_value = "127.0.0.1")]
	pub ip: Vec<String>,
	/// Port to bind to
	#[structopt(short, long, default_value = "8088")]
	pub port: usize
}