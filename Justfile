# Format the entire workspace
fmt:
    cargo fmt --all

# Regenerate SeaORM entities from the database schema
# (requires DATABASE_URL pointing to a migrated database)
gen-entities:
    sea-orm-cli generate entity -o ./crates/malprobe-database/src/model \
        --with-serde both \
        --enum-extra-derives Hash \
        --enum-extra-derives strum::Display \
        --enum-extra-derives strum::EnumString
