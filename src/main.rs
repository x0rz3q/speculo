extern crate clap;

use clap::{App, Arg, SubCommand};
use std::{
	env, fs,
	path::{Path, PathBuf},
	process::{exit, Command},
	time::{SystemTime, UNIX_EPOCH},
};

fn main() {
	let mut app = App::new("speculo")
		.version("0.2.1")
		.author("x0rz3q <jacob@x0rz3q.com>")
		.about("Mirror git repositories")
		.arg(
			Arg::with_name("path")
				.env("SPECULO_PATH")
				.required(true)
				.short("p")
				.long("path"),
		)
		.subcommand(
			SubCommand::with_name("add")
				.about("Add a base repository")
				.arg(
					Arg::with_name("repo")
						.help("Repository URL (ssh)")
						.required(true),
				)
				.arg(
					Arg::with_name("name")
						.help("Name of folder")
						.required(false),
				),
		)
		.subcommand(
			SubCommand::with_name("mirror")
				.about("Mirror a repository")
				.arg(
					Arg::with_name("base")
						.help("The base repository name")
						.required(true),
				)
				.arg(
					Arg::with_name("repo")
						.help("Repository URL (ssh)")
						.required(true),
				),
		)
		.subcommand(
			SubCommand::with_name("unlink")
				.about("Unlink a mirrored repository")
				.arg(
					Arg::with_name("base")
						.help("The base repository name")
						.required(true),
				)
				.arg(
					Arg::with_name("url")
						.help("The url to unlink")
						.required(true),
				),
		)
		.subcommand(
			SubCommand::with_name("push")
				.about("Push repositories")
				.arg(
					Arg::with_name("name")
						.help("The name of the repository to push")
						.required(false),
				),
		)
		.subcommand(
			SubCommand::with_name("rm").about("Remove repository").arg(
				Arg::with_name("name")
					.help("The name of the repository to delete")
					.required(true),
			),
		);

	let matches = app.clone().get_matches();
	let path = matches.value_of("path").unwrap();

	if !Path::new(&path).exists() {
		println!("Please make sure that SPECULO_PATH={} exists", path);
		exit(1);
	}
	env::set_current_dir(&path).unwrap();

	match matches.subcommand_name() {
		Some("add") => {
			let args = matches.subcommand_matches("add").unwrap();
			let repo = args.value_of("repo").unwrap().to_string();
			let name = match args.value_of("name") {
				Some(name) => name.to_string(),
				None => "".to_string(),
			};

			add(repo, name);
		},
		Some("mirror") => {
			let args = matches.subcommand_matches("mirror").unwrap();
			let base = args.value_of("base").unwrap().to_string();
			let repo = args.value_of("repo").unwrap().to_string();

			mirror(base, repo);
		},
		Some("push") => {
			let args = matches.subcommand_matches("push").unwrap();
			let name = args.value_of("name");

			if name.is_some() {
				push(expand_name(name.unwrap().to_string()));
			} else {
				push_all();
			}
		},
		Some("unlink") => {
			let args = matches.subcommand_matches("unlink").unwrap();
			let url = args.value_of("url").unwrap().to_string();
			let base = args.value_of("base").unwrap().to_string();

			unlink(base, url);
		},
		Some("rm") => {
			let args = matches.subcommand_matches("rm").unwrap();
			let name = args.value_of("name").unwrap();

			remove(name.to_string());
		},
		_ => {
			app.print_help().unwrap();
			println!("");
		},
	};
}

/// Expand a name to a file path
///
/// name - The name to expand
fn expand_name(name: String) -> PathBuf {
	let mut path = env::current_dir().unwrap();
	path.push(name);

	path
}

fn is_bare_repository(path: String) -> bool {
	let output = Command::new("sh")
		.arg("-c")
		.arg(format!(
			"git --git-dir=\"{path}\" --work-tree=\"{path}\" rev-parse --is-bare-repository",
			path = path
		))
		.output()
		.unwrap();

	if !output.status.success() {
		return false;
	}

	let result: String = std::str::from_utf8(&output.stdout).unwrap().to_string();
	result.contains("false")
}

/// Add master repository to the speculo store.
///
/// url - The repository URL
/// name - The name of the local folder
fn add(url: String, name: String) {
	let mut name = name;
	if name.is_empty() {
		let split = url
			.split("/")
			.collect::<Vec<&str>>()
			.last()
			.cloned()
			.unwrap();
		let file = Path::new(split);
		name = file.file_stem().unwrap().to_str().unwrap().to_string();
	}

	if Path::new(&name).exists() {
		println!("Repository {} already exists", name);
		exit(1);
	}

	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("git clone --mirror {} {}", url, name))
		.output()
		.expect("Clone of failed");

	if output.status.success() {
		println!("Successfully cloned into {}", name);
	} else {
		print!("{}", std::str::from_utf8(&output.stderr).unwrap());
	}
}

/// Remove a repository
///
/// name - The name of the repository to remove
fn remove(name: String) {
	let path = expand_name(name.clone());
	let path_str = path.to_str().unwrap().to_string();

	if !is_bare_repository(path_str) {
		println!("{} is not a git repo", name);
		exit(1);
	}

	if !path.exists() {
		println!("{} does not exist", name);
		exit(1);
	}

	if std::fs::remove_dir_all(path).is_ok() {
		println!("{} successfully removed", name);
	} else {
		println!("{} could not be removed", name);
	}
}

/// Add a mirror of a master repository.
///
/// base - The base name of the repository to mirror
/// repo - The repository end point
fn mirror(base: String, repo: String) {
	if !Path::new(&base).exists() {
		println!("{} does not exist", base);
		exit(1);
	}

	env::set_current_dir(&base).unwrap();
	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("git remote add mirror-{} {}", now, repo))
		.output()
		.expect("Clone failed!");

	if output.status.success() {
		println!("{} added as mirror to {}", repo, base);
	} else {
		print!("{}", std::str::from_utf8(&output.stderr).unwrap());
	}
}

fn push(path: PathBuf) {
	// check if the repository is a git mirror
	env::set_current_dir(path.clone()).unwrap();

	if !is_bare_repository(path.clone().to_str().unwrap().to_string()) {
		return ();
	}

	// update the master repo
	let output = Command::new("sh")
		.arg("-c")
		.arg("git remote update origin")
		.output()
		.unwrap();

	if !output.status.success() {
		print!("{}", std::str::from_utf8(&output.stderr).unwrap());
		return ();
	}

	// prune the master repo
	let output = Command::new("sh")
		.arg("-c")
		.arg("git remote prune origin")
		.output()
		.unwrap();

	if !output.status.success() {
		print!("{}", std::str::from_utf8(&output.stderr).unwrap());
		return ();
	}

	// update the mirrors
	let output = Command::new("sh")
		.arg("-c")
		.arg("git remote | grep 'mirror' | xargs -L1 git push --mirror")
		.output()
		.expect("Pushing failed!");

	if !output.status.success() {
		println!(
			"Push failed for {}",
			path.file_name().unwrap().to_str().unwrap()
		);
		print!("{}", std::str::from_utf8(&output.stderr).unwrap());
	}
}

/// Push all changes from the master repositories to the mirror
/// repositories.
fn push_all() {
	for entry in fs::read_dir(env::current_dir().unwrap()).unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		let metadata = fs::metadata(&path).unwrap();

		if !metadata.is_dir() {
			continue;
		}

		push(path);
	}
}

fn unlink(base: String, url: String) {
	// git remote -v | grep "git@git.xoryo.nl:x0rz3q/speculo.gitz" | cut -f1 |
	// head
	//-n1
	let path = expand_name(base.clone());
	let path_str = path.to_str().unwrap().to_string();

	if !is_bare_repository(path_str) {
		println!("{} is not a repository", base);
		exit(1);
	}

	env::set_current_dir(&path).unwrap();
	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("git remote -v | grep {} | cut -f1 | head -n1", url))
		.output()
		.expect("Cannot fetch repository remotes");

	if !output.status.success() {
		println!("Unlinking failed for {} in {}", url, base);
		exit(1);
	}

	let mirror: String = std::str::from_utf8(&output.stdout)
		.unwrap()
		.trim()
		.to_string();

	if mirror.is_empty() {
		println!("{} not found in {}", url, base);
		exit(1);
	}

	if mirror == "origin".to_string() {
		println!("Cannot remove origin!");
		exit(1);
	}

	let output = Command::new("sh")
		.arg("-c")
		.arg(format!("git remote remove {}", mirror))
		.output()
		.expect("Cannot remove remote");

	if !output.status.success() {
		println!("Cannot unlink {} from {}", url, base);
		exit(1);
	}
}
