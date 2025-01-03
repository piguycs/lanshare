set dotenv-load := true

@run-client:
    cargo run -p ls-client

@run-daemon:
    cargo build -p ls-daemon && pkexec target/debug/ls-daemon

@run-server:
    cargo run -p relay-server

@echo-env:
    echo $CERT_PATH
    echo $KEY_PATH

@test:
    cargo nextest run
