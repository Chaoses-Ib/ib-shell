param(
    [switch]$r
)

$p = "debug"
if ($r) {
    $p = "release-debug"
}

$buildProfile = $p
if ($p -eq "debug") {
    $buildProfile = "dev"
}
cargo build -p ib-shell --example hook --profile $buildProfile
if (!$?) {
    exit $?
}
cargo run -p ib-shell --bin inject-hook --features=hook,hook-dll,everything,bin -- --profile $p
