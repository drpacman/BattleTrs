cd battletris-web
trunk build
cd ..
cargo run -p battletris-server -- serve --web-dir battletris-web/dist

