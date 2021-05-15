target/release/citron roms/Tetris.gb &
app_pid=$!

sudo dtrace -x ustackframes=100 -n "profile-97 /pid == $app_pid/ { @[ustack()] = count(); } tick-60s { exit(0); }"  -o out.user_stacks
dtrace_pid=$!

fg

cat out.user_stacks | inferno-collapse-dtrace > stacks.folded

cat stacks.folded | inferno-flamegraph > flamegraph.svg

open flamegraph.svg