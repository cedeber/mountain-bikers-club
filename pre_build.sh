# CC_PRE_BUILD_HOOK
echo "--- Diesel & DB ---"
cargo install diesel_cli --no-default-features --features postgres
echo "DATABASE_URL=$POSTGRESQL_ADDON_URI" > .env
diesel setup
diesel migration run
echo "--- Migration done ---"