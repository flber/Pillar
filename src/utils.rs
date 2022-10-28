pub mod format {
	pub fn fast_zip(a: Vec<&str>, b: Vec<&str>) -> String {
		let mut ab: Vec<Vec<&str>> = vec![a, b];
		let mut out_bytes: Vec<u8> = vec![];

		let len = ab[0].len() + ab[1].len();
		for i in 0..len {
			let list = (i + 2) % 2;
			for c in ab[list].remove(0).bytes() {
				out_bytes.push(c)
			}
		}

		String::from_utf8_lossy(&out_bytes).to_string()
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
