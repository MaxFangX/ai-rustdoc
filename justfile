alias ci := watch-local-ci

just-fmt:
    just --fmt --unstable

watch-local-ci:
    cargo watch \
        --watch . \
        --why \
        --ignore 'glic-dist\' \
        --shell "just local-ci"

local-ci:
    cargo clippy --workspace --all-targets -- --deny=warnings
    cargo fmt --all -- --check
    cargo test
    cargo doc --document-private-items

# Continuously iterate on something
iterate *args:
    # e.g. `just iterate cargo test test_parse_individual -- --show-output`
    cargo watch \
        --watch . \
        --why \
        --shell "{{ args }}" \
        --shell "just local-ci"
