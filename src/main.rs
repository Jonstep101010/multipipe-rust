use std::{
	io::pipe,
	process::{Command, Stdio},
};

fn main() -> Result<(), std::io::Error> {
	let args: Vec<_> = std::env::args().skip(1).collect();

	let commands = get_cmds(args);
	if commands.is_empty() {
		eprintln!("No commands provided");
		return Ok(());
	}

	let mut children = Vec::new();
	let mut prev_reader = None;

	for (i, cmd_parts) in commands.iter().enumerate() {
		children.push(
			Command::new(&cmd_parts[0])
				.args(&cmd_parts[1..])
				.stdin(if let Some(reader) = prev_reader.take() {
					reader
				} else {
					// first
					Stdio::inherit()
				})
				.stdout(
					if let Some((reader, writer)) = (i != commands.len() - 1).then_some(pipe()?) {
						prev_reader = Some(reader.into());
						writer.into()
					} else {
						// last
						Stdio::inherit()
					},
				)
				.spawn()?,
		)
	}

	// parent: Wait for all processes to complete
	for mut child in children {
		child.wait()?;
	}

	Ok(())
}

///
/// Parse commands separated by "|"
fn get_cmds(args: Vec<String>) -> Vec<Vec<String>> {
	let mut commands = Vec::new();
	let mut current_cmd = Vec::new();
	for (pos, arg) in args.iter().enumerate() {
		if arg == "|" {
			// preserve previous command
			if !current_cmd.is_empty() {
				commands.push(current_cmd.clone());
				current_cmd.clear();
			}
		} else {
			current_cmd.push(arg.clone());
			// if last, preverve
			if pos + 1 == args.len() {
				commands.push(current_cmd.clone());
			}
		}
	}
	commands
}
