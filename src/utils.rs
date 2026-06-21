pub fn format_bytes(bytes: u64) -> String {
  let mb = bytes as f64 / 1_048_576.0;
  format!("{:.1} MB", mb)
}
