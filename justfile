default:
  just --list

# Check for incompatible licenses and security advisories, lint and run tests
check:
  cargo deny check
  cargo clippy
  cargo test

# Profile the appropiate benchmark
profile SUITE BENCH:
  cargo bench --bench {{SUITE}} -- --profile-time 60 {{BENCH}}

# Dynamically build binary with selected PROFILE
dbuild PROFILE:
  cargo build --profile {{PROFILE}}

# Statically build the binary with the selected PROFILE (uses musl)
sbuild PROFILE:
  RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=yes" cargo build --profile {{PROFILE}} --target x86_64-unknown-linux-musl

# Build binary in optimized mode, with CPU native optimizations. Might make the binary incompatible for older CPUs
native:
  RUSTFLAGS="-C target-cpu=native" cargo build --profile optimized
