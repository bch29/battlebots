#!/bin/fish

cargo build; and begin
set_color yellow
echo "  OUTPUT:"
set_color normal
env TESTPRG=target/debug/example cargo run --bin battlebots
end
