use std::{
	io::pipe,
	process::{Command, Stdio},
};

fn main() -> Result<(), std::io::Error> {
	let args: Vec<_> = std::env::args().skip(1).collect();

	// Parse commands separated by "|"
	let mut commands = Vec::new();
	let mut current_cmd = Vec::new();

	for arg in args {
		if arg == "|" {
			if !current_cmd.is_empty() {
				commands.push(current_cmd.clone());
				current_cmd.clear();
			}
		} else {
			current_cmd.push(arg);
		}
	}
	if !current_cmd.is_empty() {
		commands.push(current_cmd);
	}

	if commands.is_empty() {
		eprintln!("No commands provided");
		return Ok(());
	}

	let mut children = Vec::new();
	let mut prev_reader = None;

	for (i, cmd_parts) in commands.iter().enumerate() {
		let cmd_name = &cmd_parts[0];
		let cmd_args = &cmd_parts[1..];

		let mut command = Command::new(cmd_name);
		command.args(cmd_args);

		// Set up stdin
		if let Some(reader) = prev_reader.take() {
			// This is not the first command, use the previous pipe
			command.stdin(reader);
		} else {
			// First command reads from stdin
			command.stdin(Stdio::inherit());
		}

		// Set up stdout
		if i == commands.len() - 1 {
			// Last command writes to stdout
			command.stdout(Stdio::inherit());
		} else {
			// Not the last command, create a pipe
			let (reader, writer) = pipe()?;
			command.stdout(writer);
			prev_reader = Some(reader);
		}

		// Spawn the process
		let child = command.spawn()?;
		children.push(child);
	}

	// parent: Wait for all processes to complete
	for mut child in children {
		child.wait()?;
	}

	Ok(())
}
