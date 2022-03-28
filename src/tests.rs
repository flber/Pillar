#[cfg(test)]
mod tests {
  use crate::utils;

  #[test]
  fn invalid_blocks() {
    let input = r#"[invalid block [a, href: "normal block"|this one is normal]]"#;
    let expected = r#"[invalid block <a href="normal block">this one is normal</a>]"#;
    assert_eq!(
      expected.to_string(),
      utils::granite::parse_granite(&input.to_string(), false)
    )
  }
}
