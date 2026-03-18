param(
    [switch]$r
)

$p = "debug"
if ($r) {
    $p = "release-debug"
}

cargo build --example hook --profile $p
if (!$?) {
    exit $?
}
cargo run --bin inject-hook --features=hook,hook-dll,everything,bin -- --profile $p
