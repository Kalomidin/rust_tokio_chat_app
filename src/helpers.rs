pub fn get_env(key: &str) -> String {
  match std::env::var(key) {
    Ok(val) => val,
    Err(e) => panic!("couldn't interpret {}: {}", key, e),
  }
}
