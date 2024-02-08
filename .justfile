default:
  cargo build

run:
  TRACING=trace cargo run

test:
  cargo test

opt:
  cargo run --release

fix:
  cargo clippy --fix

check:
 cargo clippy

clean:
  cargo clean
  rm ./crates/render/shaders/spv/*.spv

alias c := clean
alias r := run
