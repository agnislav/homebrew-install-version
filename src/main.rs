use std::fmt;
use std::io::{Read, Write};
use clap::{Arg, Command};
use tempfile::{Builder, NamedTempFile, TempDir};

// #[tokio::main]
fn main() {
    let (formula_name, formula_version) = parse_arguments();
    Formula::new(formula_name, formula_version).init();
}

fn parse_arguments() -> (String, String) {
    let matches = Command::new("homebrew_install_version")
        .version("1.0")
        .author("Your Name")
        .about("Installs a specific version of a Homebrew formula")
        .arg(Arg::new("formula_name")
            .help("The name of the formula")
            .required(true)
            .index(1))
        .arg(Arg::new("formula_version")
            .help("The version of the formula")
            .required(true)
            .index(2))
        .get_matches();

    let formula_name = matches.get_one::<String>("formula_name").unwrap().to_string();
    let formula_version = matches.get_one::<String>("formula_version").unwrap().to_string();

    (formula_name, formula_version)
}

pub struct Formula {
    name: String,
    version: String,
    repo_path: Option<String>,
    commit: Option<String>,
    url: Option<String>,
    temp_dir: Option<TempDir>,
    bottle_file: Option<NamedTempFile>,
}

impl Formula {
    fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            repo_path: None,
            commit: None,
            url: None,
            temp_dir: None,
            bottle_file: None,
        }
    }

    fn init(&mut self) -> &mut Self {
        self.get_commit_hash().download().install();
        println!("Formula: {:?}", self);
        self
    }

    fn get_file_path(&self) -> [String; 2] {
        let first_letter = self.name.chars().next().unwrap();
        [format!("/Formula/{}/{}.rb", first_letter, self.name), format!("/Formula/{}.rb", self.name)]
    }

    fn get_commit_hash(&mut self) -> &mut Self {
        for file_path in self.get_file_path() {
            let url = format!("https://api.github.com/repos/Homebrew/homebrew-core/commits?path={}&per_page=100", file_path);
            println!("URL: {:?}", &url);
            let client = reqwest::blocking::Client::new();
            let response = client.get(&url)
                .header("User-Agent", "homebrew_install_version")
                .send()
                .unwrap();

            let json: serde_json::Value = response.json().unwrap();
            for commit in json.as_array().unwrap() {
                let commit_message = commit.get("commit").unwrap().get("message").unwrap().as_str().unwrap().to_string();
                println!("{}: {}", file_path, commit_message);
                if commit_message.contains(&format!("{}: update {} bottle", self.name, self.version)) {
                    println!("Found Commit: {:?}", commit.get("sha").unwrap().as_str().unwrap().to_string());
                    let commit_hash = Some(commit.get("sha").unwrap().as_str().unwrap().to_string());
                    self.commit = commit_hash;
                    self.url = Some(format!("https://raw.githubusercontent.com/Homebrew/homebrew-core/{}{}", self.commit.clone().unwrap(), file_path));
                    self.repo_path = Some(file_path);
                    break;
                }
            }
        }
        self
    }

    fn download(&mut self) -> &mut Self {
        let client = reqwest::blocking::Client::new();
        let response = client.get(self.url.clone().unwrap())
            .header("User-Agent", "homebrew_install_version")
            .send()
            .unwrap();

        let file_content = response.text().unwrap();

        // create temp file
        let tmp_dir = Builder::new().tempdir().unwrap();
        let mut temp_file = Builder::new()
            .prefix(&self.name)
            .suffix(".rb")
            .rand_bytes(0)
            .tempfile_in(tmp_dir.path())
            .unwrap();

        println!("Temp File: {:?}", &temp_file.path());

        let res = temp_file.write_all(file_content.as_bytes());
        match res {
            Ok(_) => {
                self.temp_dir = Some(tmp_dir);
                self.bottle_file = Some(temp_file);
            }
            Err(e) => {
                println!("Error writing to temp file: {:?}", e);
            }
        }
        self
    }

    fn install(&mut self) -> &mut Self {
        let mut cmd = std::process::Command::new("brew");
        cmd.arg("remove");
        cmd.arg(&self.name);
        let output = cmd.output();
        match output {
            Ok(output) => println!("output: {:?}", output),
            Err(e) => println!("Error running brew remove: {:?}", e),
        }

        println!("Install from File: {:?}", &self.bottle_file.as_ref().unwrap().path());

        let mut file = std::fs::File::open(self.bottle_file.as_ref().unwrap().path()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        println!("Bottle File Content: {}", contents);

        let mut cmd = std::process::Command::new("brew");
        cmd.arg("install");
        cmd.arg(self.bottle_file.as_ref().unwrap().path());
        println!("COMMAND: {:?}", cmd);
        let output = cmd.output();
        match output {
            Ok(output) => println!("output: {:?}", output),
            Err(e) => println!("Error running brew install with custom formula: {:?}", e),
        }
        self
    }
}

impl fmt::Debug for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Formula: {}, Version: {}, Commit: {:?}, URL: {:?}", self.name, self.version, self.commit, self.url)
    }
}