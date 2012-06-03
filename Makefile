RUSTC ?= rustc

# ------------------
# Internal variables
dummy1 := $(shell mkdir bin 2> /dev/null)

# ------------------
# Primary targets
all: lib

#check: bin/test-rrest
#	export RUST_LOG=rrest=1,rparse=1 && ./bin/test-rrest

# Better to use /usr/local/lib but linking it in with -L /usr/local/lib fails because
# there is a libccore there and in the nested rustc directory.
install: lib
	install -p `find bin -name "librrdf*" -type f -maxdepth 1` /usr/local/lib/rust

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
	$(RUSTC) --out-dir bin -O src/rrdf.rc

#bin/test-rrest: src/rrest.rc src/*.rs
#	$(RUSTC) -g -L /usr/lib --test -o $@ $<
