pub mod format {
	pub fn fast_zip(xs: Vec<&str>, ys: Vec<&str>) -> String {
		coz::scope!("fast_zip");
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
		zipped.push(String::from(extra));

		zipped.join("")
	}
}

#[allow(dead_code)]
pub mod bench {
	use crate::utils::math;
	use std::time::Instant;

	pub fn average<F>(f: F, iter: u64) -> (u64, u32)
	where
		F: Fn(),
	{
		let mut times: Vec<u64> = vec![iter; 0];
		for _ in 0..iter {
			let t = time(&f);
			times.push(t);
		}
		let len = times.len() as u64;
		let average = times.iter().sum::<u64>() / len;
		let deviation =
			math::std_deviation(&times.iter().map(|x| *x as i32).collect::<Vec<i32>>()[..])
				.unwrap();

		(average, deviation as u32)
	}

	pub fn time<F>(f: F) -> u64
	where
		F: Fn(),
	{
		let start = Instant::now();
		f();
		let nano: u64 = start.elapsed().as_nanos() as u64;

		nano /* as f64 / 1_000_000_000.0 */
	}
}

pub mod math {
	pub fn mean(data: &[i32]) -> Option<f32> {
		let sum = data.iter().sum::<i32>() as f32;
		let count = data.len();

		match count {
			positive if positive > 0 => Some(sum / count as f32),
			_ => None,
		}
	}

	pub fn std_deviation(data: &[i32]) -> Option<f32> {
		match (mean(data), data.len()) {
			(Some(data_mean), count) if count > 0 => {
				let variance = data
					.iter()
					.map(|value| {
						let diff = data_mean - (*value as f32);

						diff * diff
					})
					.sum::<f32>() / count as f32;

				Some(variance.sqrt())
			}
			_ => None,
		}
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

	#[test]
	fn test_math_deviation() {
		assert_eq!(
			2.9832866,
			math::std_deviation(&[9, 2, 5, 4, 12, 7, 8, 11, 9, 3, 7, 4, 12, 5, 4, 10, 9, 6, 9, 4])
				.unwrap()
		);
		assert_eq!(3.304038, math::std_deviation(&[9, 2, 5, 4, 12, 7]).unwrap());
		assert_eq!(
			4.8989797,
			math::std_deviation(&[10, 12, 23, 23, 16, 23, 21, 16]).unwrap()
		);
		assert_eq!(
			164454.67,
			math::std_deviation(&[10123, 456745, 1234, 14356, 76547, 2345]).unwrap()
		);
	}
}
