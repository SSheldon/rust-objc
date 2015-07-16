ARCHS ?= i386 x86_64 armv7 armv7s arm64
RUST_ARCHS = $(subst arm64,aarch64,$(ARCHS))
TARGETS = $(addsuffix -apple-ios,$(RUST_ARCHS))
LIB = libobjc_tests.a

all: $(LIB)

.PHONY: $(TARGETS)
$(TARGETS): %:
	cargo build --target $@

$(LIB): $(TARGETS)
	lipo -create -output $@ $(foreach arch,$(TARGETS),$(wildcard target/$(arch)/debug/$(LIB)))
