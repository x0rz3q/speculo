use std::env;
use std::process::exit;
use std::path::Path;
use std::process::Command;

fn print_usage() {
	println!("Hello");
}

fn main() {
	let args: Vec<String> = env::args().collect();

	if args.len() == 1 {
		print_usage();
		exit(1);
	}

	let command: String = args.get(1).cloned().unwrap();
	let path = match env::var("SPECULO_PATH") {
		Ok(path) => path,
		Err(_) => {
			println!("Please set the SPECULO_PATH env variable");
			exit(1);
		}
	};

	if ! Path::new(&path).exists() {
		println!("{} does not exist, please set SPECULO_PATH correctly", path);
		exit(1);
	}
	env::set_current_dir(&path).unwrap();

	match command.as_ref() {
		"add" => add(args),
		_ => {
			print_usage();
			exit(1);
		}
	};
}

fn add(args: Vec<String>) {
	if args.len() < 3 {
		print_usage();
		exit(1);
	}

	let url_str = args.get(2).cloned().unwrap();
	let mut name: String;

	if args.len() == 4 {
		name = args.get(3).cloned().unwrap();
	} else {
		let split = url_str.split("/").collect::<Vec<&str>>().last().cloned().unwrap();
		let file = Path::new(split);
		name = file.file_stem().unwrap().to_str().unwrap().to_string();
	}

	if Path::new(&name).exists() {
		println!("Repository {} already exists", name);
		exit(1);
	}

	let output = Command::new("sh")
			.arg("-c")
			.arg(format!("git clone {} {}", url_str, name))
			.output()
			.expect("Clone of failed");

	if output.status.success() {
		println!("Successfully cloned into {}", name);
	} else {
		println!("{}", std::str::from_utf8(&output.stderr).unwrap());
	}
}
