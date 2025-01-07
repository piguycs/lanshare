set dotenv-load := true

@run-server:
    cargo run -p relay-server

@run-client:
    cargo run -p ls-client

@run-daemon-root:
    cargo build -p ls-daemon && pkexec target/debug/ls-daemon

@test:
    cargo nextest run

clean-socks:
    ls $XDG_RUNTIME_DIR/lanshare-*
    rm $XDG_RUNTIME_DIR/lanshare-*
    ls /run/lanshare
    pkexec rm -r /run/lanshare
