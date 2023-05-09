use std::io::{self, Write};

pub struct Bar {
	pub left_pad: usize,
	pub bar_width: usize,
	pub max: usize,
}

impl Bar {
	pub fn print(&self, i: usize) {
		let inverse = 1.0 / (i as f32 / self.max as f32);
		let progress = (self.bar_width as f32 / inverse) as usize;
		let percent = if (100.0 / inverse).ceil() > 100.0 {
			100.0
		} else {
			(100.0 / inverse).ceil()
		};

		if self.bar_width >= progress {
			print!(
				"\r{:#left$} [{:=>mid$}{:->right$}",
				percent,
				">",
				"]",
				left = self.left_pad,
				mid = progress,
				right = self.bar_width - progress
			);
		}
		io::stdout().flush().unwrap();
	}
}

pub fn terminal_size() -> Option<(u16, u16)> {
	use std::process::Command;
	use std::process::Stdio;

	let output = Command::new("stty")
		.arg("size")
		.arg("-F")
		.arg("/dev/stderr")
		.stderr(Stdio::inherit())
		.output()
		.unwrap();

	let stdout = String::from_utf8(output.stdout).unwrap();
	if !output.status.success() {
		return None;
	}

	// stdout is "rows cols"
	let mut data = stdout.split_whitespace();
	let rows = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
	let cols = u16::from_str_radix(data.next().unwrap(), 10).unwrap();
	Some((rows, cols))
}
