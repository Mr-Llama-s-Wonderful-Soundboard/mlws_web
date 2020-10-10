use mlws_lib::config::Config;

use server_client::server_client;
server_client!(
	pub Config {
		fn load() -> Config {
			Config::load()
		}

		fn save(conf: Config) {
			conf.save()
		}
	}
);