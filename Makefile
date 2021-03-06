RUSTC ?= rustc

# ------------------
# Internal variables
dummy1 := $(shell mkdir bin 2> /dev/null)

# ------------------
# Primary targets
all: lib

check: bin/test-rrdf
	export RUST_LOG=rrdf=1,rparse=1,::rt::backtrace=4 && ./bin/test-rrdf

# Logging seems all screwed up: if you want to see rparse logs use r=2
check1: bin/test-rrdf
	export RUST_LOG=rrdf::query=2,rrdf::expression=1,rparse=1,::rt::backtrace=4 && ./bin/test-rrdf select_all

speed: bin/test-speed
	export RUST_LOG=rrdf::query=1 && ./bin/test-speed speed

# You can either use this target (assuming that the libraries are in /usr/local/lib/rust)
# or install them via cargo.
update-libraries:
	cp /usr/local/lib/rust/librparse-*-0.6.* bin

# Better to use /usr/local/lib but linking it in with -L /usr/local/lib fails because
# there is a libccore there and in the nested rustc directory.
install:
	install -p `find bin -maxdepth 1 -name "librrdf*" -type f` /usr/local/lib/rust

clean:
	rm -rf bin

# ------------------
# Binary targets 
# We always build the lib because:
# 1) We don't do it that often.
# 2) It's fast.
# 3) The compiler gives it some crazy name like "librrrest-da45653350eb4f90-0.1.dylib"
# which is dependent on some hash(?) as well as the current platform. (And -o works when
# setting an executable's name, but not libraries).
.PHONY : lib
lib:
	$(RUSTC) -L bin --out-dir bin -O src/rrdf.rc

bin/test-rrdf: src/rrdf.rc src/*.rs src/tests/*.rs
	$(RUSTC) -g -L bin --test -o $@ $<

bin/test-speed: src/rrdf.rc src/*.rs src/tests/*.rs src/bench/*.rs
	$(RUSTC) -g -L bin --cfg speed -O --test -o $@ $<
