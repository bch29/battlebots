#!/bin/fish

cargo build --release; and begin
set_color yellow
echo "  OUTPUT:"
set_color normal
env TESTPRG=target/debug/example cargo run --release --bin battlebots
end
