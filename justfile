set dotenv-load := true

@default:
    just --list

@run-server:
    cargo run -p relay-server

@run-client:
    cargo run -p ls-client

@run-daemon-root:
    cargo build -p ls-daemon && sudo -E target/debug/ls-daemon

@test:
    cargo nextest run

@coverage:
    cargo llvm-cov nextest

# ehhh, I should prolly impliment Drop and handle these there
#clean-socks:
#    ls $XDG_RUNTIME_DIR/lanshare-*
#    rm $XDG_RUNTIME_DIR/lanshare-*
#    ls /run/lanshare
#    pkexec rm -r /run/lanshare
