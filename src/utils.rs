pub mod format {
	use std::cmp;

	pub fn fast_zip(xs: Vec<&str>, ys: Vec<&str>) -> String {
		let mut extra = "";
		if xs.len() != ys.len() {
			if xs.len() > ys.len() {
				extra = xs[xs.len() - 1];
			} else {
				extra = ys[ys.len() - 1];
			}
		}

		let mut zipped = xs
			.iter()
			.zip(ys.iter())
			.map(|(a, b)| vec![*a, *b].concat())
			.collect::<Vec<String>>();

		/*
			let len = cmp::min(xs.len(), ys.len());
			let xs = &xs[..len];
			let ys = &ys[..len];

			let mut zipped: Vec<&str> = vec![];
			for i in 0..len {
				let x = xs[i];
				let y = ys[i];

				zipped.push(x);
				zipped.push(y);
			}
		*/

		zipped.push(String::from(extra));

		zipped.join("")
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_fast_zip_uneven() {
		let data = vec!["A", "B", "C"];
		let separators = vec![" says hello to ", " but not to "];

		let flat = format::fast_zip(data, separators);
		assert_eq!("A says hello to B but not to C", flat);
	}

	#[test]
	fn test_fast_zip_even() {
		let data = vec!["A", "B"];
		let separators = vec![" says hello to ", " and that's it"];

		let flat = format::fast_zip(data, separators);
		assert_eq!("A says hello to B and that's it", flat);
	}
}
