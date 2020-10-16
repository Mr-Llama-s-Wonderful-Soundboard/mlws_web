use crate::opts::SoundboardOpt;
use git2::Repository;
use std::fs::{copy, create_dir_all, File};
use std::process::Command;
use std::{io::Write, path::PathBuf};

const GITHUB_ACTIONS: &'static str = r#"# This is a basic workflow to help you get started with Actions

name: Release zip

on: push

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: sudo apt install zip
      - name: Zip files
        run: CURR_DIR="$(basename $PWD)" && cd .. && mkdir data && mv $CURR_DIR/* ./data/ && zip -r sounds.zip data && cd "./$CURR_DIR" && mv ../sounds.zip ./
      - name: Add version file
        run: git rev-parse --short HEAD > version.txt
      - name: Delete tag and release
        uses: dev-drprasad/delete-tag-and-release@v0.1.2
        with:
          tag_name: latest
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      - name: Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: latest
          prerelease: true
          files: |
            version.txt
            sounds.zip
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}"#;
const README: &'static str = r#"Sounds repository for [Mr Llama's Wonderful Soundboard](https://github.com/Mr-Llama-s-Wonderful-Soundboard/mlws_web)"#;

pub fn handle(opts: SoundboardOpt) {
    match opts {
        SoundboardOpt::Create {
            git,
            github_actions,
            commit,
            push,
            path,
            name,
            default_img,
        } => {
            let path = path.unwrap_or(PathBuf::from(&name));
            if path.exists() {
                println!("Path {} already exists", path.display())
            } else {
                if default_img.exists() && default_img.is_file() {
                    let git = git.unwrap_or(true);
                    let github_actions = github_actions.unwrap_or(true);
                    let mut spec = Vec::new();
                    create_dir_all(&path).expect("Error creating main directory");
                    if github_actions {
                        create_dir_all(&path.join(".github/workflows"))
                            .expect("Error creating github actions directory");
                        let mut f = File::create(&path.join(".github/workflows/main.yml"))
                            .expect("Error creating github actions workflow");
                        write!(f, "{}", GITHUB_ACTIONS)
                            .expect("Error writing github actions workflow");
                        spec.push(".github/workflows/main.yml".to_string())
                    }
                    let mut sounds =
                        File::create(&path.join("sounds.ron")).expect("Error creating sounds.ron");
                    write!(
                        sounds,
                        r#"(default_img: {:?}, name: "{}", sounds: [])"#,
                        default_img.file_name().unwrap(),
                        name
                    )
                    .expect("Error writing sounds.ron");
                    spec.push("sounds.ron".to_string());
                    let mut readme =
                        File::create(&path.join("README.md")).expect("Error creating README.md");
                    write!(readme, "# {}\n{}", name, README).expect("Error writing sounds.ron");
                    spec.push("README.md".to_string());
                    copy(&default_img, &path.join(&default_img.file_name().unwrap())).unwrap();
                    spec.push(
                        PathBuf::from(default_img.file_name().unwrap())
                            .display()
                            .to_string(),
                    );

                    if git {
                        let _ = Repository::init(&path).expect("Error creating repo");

                        if commit || push.is_some() {
                            Command::new("git")
                                .current_dir(&path)
                                .arg("add")
                                .args(&spec)
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                            Command::new("git")
                                .current_dir(&path)
                                .arg("commit")
                                .args(&["-m", "Initial commit"])
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        }
                        if let Some(repo) = push {
                            Command::new("git")
                                .current_dir(&path)
                                .args(&["remote", "add", "origin", &repo])
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                            Command::new("git")
                                .current_dir(&path)
                                .args(&["branch", "-M", "main"])
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                            Command::new("git")
                                .current_dir(&path)
                                .args(&["push", "-u", "origin", "main"])
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        }
                    }
                } else {
                    println!("Default image doesn't exist or is a directory");
                }
            }
        }

        SoundboardOpt::Update {
            clone,
            commit,
            push,
            path,
            sounds,
        } => {
            println!("{:?}", sounds);
        }
    }
}
