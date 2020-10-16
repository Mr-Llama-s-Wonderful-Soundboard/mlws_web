use structopt::StructOpt;
use std::{path::PathBuf, str::FromStr};


#[derive(StructOpt, Debug)]
#[structopt(name = "mlws_web")]
pub struct Opt {
    /// IPs to bind to
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub ip: Vec<String>,
    /// Port to bind to
    #[structopt(short, long, default_value = "8088")]
    pub port: usize,

    #[structopt(subcommand)]
    pub subcommand: Option<Subcommmand>,
}

#[derive(StructOpt, Debug)]
pub enum Subcommmand {
    /// Create / Edit a soundboard
    Soundboard {
        #[structopt(subcommand)]
        subcommand: SoundboardOpt,
    },
}

#[derive(StructOpt, Debug)]
pub enum SoundboardOpt {
    /// Create a soundboard
    Create {
        /// Create git repo (Defaults to true)
        #[structopt(long)]
        git: Option<bool>,
        /// Use github actions (Defaults to true)
        #[structopt(long)]
        github_actions: Option<bool>,
        /// Create a commit with the email (Requires the git option & the ability to execute `git`)
        #[structopt(long)]
        commit: bool,
        /// Push a commit to an online repo (Requires the git option & the ability to execute `git`)
        #[structopt(long)]
        push: Option<String>,
		/// Path in which to create the repository (Defaults to the name)
		#[structopt(long)]
		path: Option<PathBuf>,
		/// Repository name
		name: String,
		/// Default image
		default_img: PathBuf,
    },

    Update {
        /// Clone git repo
        #[structopt(long)]
        clone: Option<String>,

        /// Commit (Requires a git repo option & the ability to execute `git`)
        #[structopt(long)]
        commit: bool,

        /// Push a commit to an online repo (Requires the git option & the ability to execute `git`)
        #[structopt(long)]
        push: Option<String>,

		/// Path in which to clone / load the repository (Defaults to the current dir)
		#[structopt(long)]
        path: Option<PathBuf>,
        /// Put the sounds in the format <name>=<sound file>[:<image file>]
        /// If using spaces, it should be in between qoutes
        sounds: Vec<Sound>
    }
}

#[derive(StructOpt, Debug)]
pub struct Sound {
    name: String,
    sound: PathBuf,
    img: Option<PathBuf>
}

impl FromStr for Sound {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let name = split.next().unwrap().trim().to_string();
        let mut split = split.next().unwrap().split(":");
        let sound = PathBuf::from_str(split.next().unwrap().trim()).unwrap();
        let img = split.next().map(|x|PathBuf::from_str(x.trim()).unwrap());
        Ok(Self{
            name, sound, img
        })
    }
}
