default_prefix = /usr/local
prefix ?= $(default_prefix)
exec_prefix = $(prefix)
bindir = $(exec_prefix)/bin
libdir = $(exec_prefix)/lib
includedir = $(prefix)/include
datarootdir = $(prefix)/share
datadir = $(datarootdir)

SOURCES = $(shell find src -type f -wholename '*src/*.rs') Cargo.toml Cargo.lock

RELEASE = debug
DEBUG ?= 0
ifeq (0,$(DEBUG))
	ARGS = --release
	RELEASE = release
endif

VENDORED ?= 0
# ifeq (1,$(VENDORED))
#     ARGS += --frozen
# endif

TARGET = target/$(RELEASE)

.PHONY: all clean distclean install uninstall update

BIN=hed

all: cli

cli: $(TARGET)/$(BIN) $(TARGET)/$(BIN).1.gz $(SOURCES) 

clean:
	cargo clean

distclean: clean
	rm -rf .cargo vendor vendor.tar

vendor: vendor.tar

vendor.tar:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcf vendor.tar vendor
	rm -rf vendor

install-cli: cli
	install -Dm 0755 "$(TARGET)/$(BIN)" "$(DESTDIR)$(bindir)/$(BIN)"
	install -Dm 0644 "$(TARGET)/$(BIN).1.gz" "$(DESTDIR)$(datadir)/man/man1/$(BIN).1.gz"

install: all install-cli

uninstall-cli:
	rm -f "$(DESTDIR)$(bindir)/$(BIN)"
	rm -f "$(DESTDIR)$(datadir)/man/man1/$(BIN).1.gz"

uninstall: uninstall-cli 

update:
	cargo update

extract:
ifeq ($(VENDORED),1)
	tar pxf vendor.tar
endif

$(TARGET)/$(BIN): 
	cargo build --manifest-path Cargo.toml $(ARGS)

$(TARGET)/$(BIN).1.gz: $(TARGET)/$(BIN)
	help2man --no-info $< | gzip -c > $@.partial
	mv $@.partial $@

deb: cli
	dpkg-buildpackage -b -rfakeroot -us -uc
