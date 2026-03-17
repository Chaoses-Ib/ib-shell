cargo build --example hook
if (!$?) {
    exit $?
}
cargo run --bin inject-hook --features=hook,hook-dll,bin
