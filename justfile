@run-client:
    cargo run -p ls-client

@run-daemon:
    cargo build -p ls-daemon && pkexec target/debug/ls-daemon
