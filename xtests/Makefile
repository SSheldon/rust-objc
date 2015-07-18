ARCHS ?= i386 x86_64 armv7 armv7s arm64
RUST_ARCHS = $(subst arm64,aarch64,$(ARCHS))
TARGETS = $(addsuffix -apple-ios,$(RUST_ARCHS))

CARGO_MODE ?= debug
LIB = libobjc_tests.a
TARGET_LIBS = $(foreach target,$(TARGETS),target/$(target)/$(CARGO_MODE)/$(LIB))

all: $(LIB)

.PHONY: $(TARGETS)
$(TARGETS):
	cargo build --target $@

target/%/$(CARGO_MODE)/$(LIB): %;

$(LIB): $(TARGET_LIBS)
	lipo -create -output $@ $(TARGET_LIBS)
