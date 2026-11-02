.PHONY: all start run clean

all:
	cargo build --release

start: run

run:
	@target/release/hospital-sim

clean:
	cargo clean

check:
	cargo check
	rg -i 'todo|fixme' --iglob='!Makefile'
	typos
