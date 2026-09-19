default:
    @echo "No default task specified"
    just --list

dkc:
    docker pull ghcr.io/leoborai/dkc:latest
    docker run -it --rm \
        -v $(pwd):/app \
        -w /app \
        ghcr.io/leoborai/dkc:latest

fmt:
    cargo clippy --fix --workspace --allow-dirty --allow-staged
    cargo fmt
