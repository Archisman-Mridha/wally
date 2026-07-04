fn main() -> Result<(), Box<dyn std::error::Error>> {
  prost_build::compile_protos(&["proto/wally/v1/wal_entry.proto"], &["proto/wally/v1"])?;

  Ok(())
}
