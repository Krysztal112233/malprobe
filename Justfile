# Format the entire workspace (Rust crates + web frontend)
fmt:
    cargo fmt --all
    pnpm --dir web run format

# Regenerate SeaORM entities from the database schema
# (requires DATABASE_URL pointing to a migrated database)
gen-entities:
    sea-orm-cli generate entity -o ./crates/malprobe-database/src/model \
        --with-serde both \
        --enum-extra-derives Hash \
        --enum-extra-derives strum::Display \
        --enum-extra-derives strum::EnumString
