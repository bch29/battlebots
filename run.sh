#!/bin/fish

cargo build; and begin
set_color yellow
echo "  OUTPUT:"
set_color normal
env TESTPRG=target/debug/testprg cargo run --bin robo2
set_color yellow
echo "  LOG:"
set_color normal
cat log.txt
end
