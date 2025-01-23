use std::{
    collections::HashSet,
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, Write},
    process::Command,
};

use serde_json;

struct Template {
    name: String,
    print_str: String,
}

const BLUE: &str = "\x1b[0;34m";
const GREEN: &str = "\x1B[0;32m";
const RESET: &str = "\x1B[0m";

pub enum NewOrInit {
    New,
    Init,
}

impl fmt::Display for NewOrInit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::New => write!(f, "new"),
            Self::Init => write!(f, "init"),
        }
    }
}

type Templates = Vec<Template>;

pub struct Cli {
    pub template: String,
    pub new_or_init: NewOrInit,
    pub project_name: String,
    pub init_git: bool,
    pub clear_readme: bool,

    url: String,
}

impl Cli {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let templates = Self::fetch_templates()?;

        Ok(Self {
            template: Self::get_template(&templates)?,
            new_or_init: Self::get_new_or_init()?,
            project_name: Self::get_project_name()?,
            init_git: Self::get_init_git()?,
            clear_readme: Self::get_clear_readme()?,

            url: String::from("github:nulladmin1/nix-flake-templates"),
        })
    }

    fn fetch_templates() -> Result<Templates, Box<dyn Error>> {
        println!("📥 Fetching templates...");

        let args = [
            "--extra-experimental-features",
            "'nix-command flakes'",
            "flake",
            "show",
            "--json",
            "github:nulladmin1/nix-flake-templates",
        ];
        let mut command = Command::new("nix");
        command.args(args);

        match command.output() {
            Ok(output) => {
                let output_json = String::from_utf8(output.stdout)?;
                let parsed_json: serde_json::Value = serde_json::from_str(&output_json)?;
                let templates_json = parsed_json.get("templates").unwrap();

                let mut templates: Templates = Vec::new();

                for (key, value) in templates_json.as_object().unwrap() {
                    let description = if key == &"default".to_owned() {
                        "Empty/Blank".to_string()
                    } else {
                        value
                            .get("description")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .strip_prefix("Nix Flake Template for ")
                            .unwrap()
                            .to_string()
                    };
                    templates.push(Template {
                        name: key.to_string(),
                        print_str: description,
                    });
                }
                let mut duplicate_descriptions: HashSet<String> = HashSet::new();
                templates
                    .retain(|template| duplicate_descriptions.insert(template.print_str.clone()));

                Ok(templates)
            }
            Err(e) => {
                eprintln!("❌ Failed to fetch templates: {e}");
                Err(Box::from("Failed to fetch templates"))
            }
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        println!("\n🫵 You selected: ");
        println!("- Template: {GREEN}{0}{RESET}", self.template);
        println!("- To {GREEN}{0}{RESET}", self.new_or_init);
        println!("- Project name: {GREEN}{0}{RESET}", self.project_name);
        println!("- Initialize Git: {GREEN}{0}{RESET}", self.init_git);
        println!("- Clear README.md: {GREEN}{0}{RESET}", self.clear_readme);

        println!("\n🚀 Initializing project...");

        let url = format!("{}#{}", self.url.as_str(), self.template.as_str());

        let new_or_init_string = self.new_or_init.to_string();
        let new_or_init = new_or_init_string.as_str();

        let args = [
            "--extra-experimental-features",
            "'nix-command flakes'",
            "flake",
            new_or_init,
            "--template",
            &url,
        ];

        let mut command_string = "nix ".to_string() + args.join(" ").as_str();

        let mut command = Command::new("nix");

        command.args(args);

        let directory = match &self.new_or_init {
            NewOrInit::New => self.project_name.clone(),
            NewOrInit::Init => ".".to_string(),
        };

        if let NewOrInit::New = self.new_or_init {
            let project_name = self.project_name.as_str();
            command.arg(project_name);
            command_string.push_str(format!(" {project_name}").as_str());
        }

        println!("❄️ Running {GREEN}{command_string}{RESET} ...");
        command.output()?;
        println!("👑 Created project {GREEN}successfully{RESET}\n");

        println!("🔀 Updating project details with the project name...");
        self.update_project_names()?;

        println!();

        if self.init_git {
            println!("🔧 Initializing Git repository...");
            Command::new("git")
                .args(["init", directory.as_str()])
                .output()?;
            println!("🔧 Initialized Git repository {GREEN}successfully{RESET}\n");
        }

        if self.clear_readme {
            println!("🧹 Clearing README.md file...");
            let mut file = File::create(format!("{}/README.md", directory.as_str()))?;
            let content = format!("# {0}\n\nLorem ipsum dolor sit amet", self.project_name);
            file.write_all(content.as_bytes())?;
            println!("🧹 Cleared README.md file {GREEN}successfully{RESET}\n");
        }

        println!("🎉 Done!");

        Ok(())
    }

    fn print_prompt() -> Result<(), Box<dyn Error>> {
        print!("> ");
        io::stdout().flush()?;
        Ok(())
    }

    fn get_template(templates: &Templates) -> Result<String, Box<dyn Error>> {
        println!("📦 What {GREEN}template{RESET} do you want to use? ");

        (1..templates.len() + 1).for_each(|i| {
            let template_str = &templates[i - 1].print_str;
            println!("  {BLUE}{i}){RESET} {template_str}");
        });
        print!("👆 Pick a number or enter the code for the template: ");
        io::stdout().flush()?;

        let mut template_input = String::new();
        io::stdin().read_line(&mut template_input)?;
        let template_input: usize = template_input
            .trim()
            .parse()
            .expect("Please enter a {GREEN}number{RESET} within that range");

        Ok(templates[template_input - 1].name.to_owned())
    }

    fn get_new_or_init() -> Result<NewOrInit, Box<dyn Error>> {
        println!("🤔 Do you want to create a {GREEN}new{RESET} project or {GREEN}init{RESET}ialize one in this folder?");
        Self::print_prompt()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        match input.trim().to_lowercase().as_str() {
            "new" | "n" => Ok(NewOrInit::New),
            "init" | "i" => Ok(NewOrInit::Init),
            _ => Err(Box::from("❌Invalid input: enter 'new' to create a new project; 'init' to initialize one in this folder")),
        }
    }

    fn get_project_name() -> Result<String, Box<dyn Error>> {
        println!("📝 What do you want to name your project?");
        Self::input_string()
    }

    fn get_init_git() -> Result<bool, Box<dyn Error>> {
        println!("💾Do you want to initialize a Git repository (using git init)?");
        Self::input_bool()
    }

    fn get_clear_readme() -> Result<bool, Box<dyn Error>> {
        println!("📄Do you want to clear the README.md file?");
        Self::input_bool()
    }

    fn input_bool() -> Result<bool, Box<dyn Error>> {
        Self::print_prompt()?;
        let mut input_string = String::new();
        io::stdin().read_line(&mut input_string)?;

        match input_string.trim().to_lowercase().as_str() {
        "y" | "yes" | "true" => Ok(true),
        "n" | "no" | "false" => Ok(false),
        _ => Err(Box::from("❌Invalid input: enter 'y', 'yes', or 'true' to agree; 'n', 'no', or 'false' to disagree")),
        }
    }

    fn input_string() -> Result<String, Box<dyn Error>> {
        Self::print_prompt()?;
        let mut input_string = String::new();
        io::stdin().read_line(&mut input_string)?;
        Ok(input_string.trim().to_owned())
    }

    fn update_project_names(&self) -> Result<(), Box<dyn Error>> {
        let directory = (match &self.new_or_init {
            NewOrInit::New => self.project_name.clone(),
            NewOrInit::Init => ".".to_string(),
        }) + "/";

        // Rename all files containing "project_name" with &self.project_name
        if let Ok(output) = Command::new("grep")
            .args(["-rl", "project_name", directory.as_str()])
            .output()
        {
            let file_names = String::from_utf8_lossy(&output.stdout);
            for file_name in file_names.lines() {
                if let Ok(content) = fs::read_to_string(file_name) {
                    if content.contains("project_name") {
                        let new_content = content.replace("project_name", &self.project_name);
                        if fs::write(file_name, new_content).is_ok() {
                            println!(
                                "- ✔️ Replaced 'project_name' placeholder with {0} in file {1}",
                                &self.project_name, &file_name
                            );
                        } else {
                            eprintln!("- ❌Failed to write to file: {file_name}");
                        }
                    }
                } else {
                    eprintln!("- ❌Failed to read file: {file_name}");
                }
            }
        } else {
            eprintln!("Unable to run 'grep' to find all instances of 'project_name' within the flake directory. ")
        }

        // Rename all files and folders containing "project_name" with &self.project_name
        if let Ok(output) = Command::new("find")
            .args([directory.as_str(), "-name", "*project_name*"])
            .output()
        {
            let paths = String::from_utf8_lossy(&output.stdout);
            for path in paths.lines() {
                let new_path = path.replace("project_name", &self.project_name);
                if fs::rename(path, &new_path).is_ok() {
                    println!(
                        "- ✔️ Renamed {0} containing 'project_name' to {1}",
                        &path, &self.project_name
                    );
                } else {
                    eprintln!("- ❌Failed to rename file or folder: {path}");
                }
            }
        } else {
            eprintln!("- ❌Failed to find files or folders containing 'project_name'.");
        }
        Ok(())
    }
}
