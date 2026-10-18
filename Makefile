# Builds, installs and packages WhatsUp.
#   Fedora-based systems, in the whatsup-desktop toolbox:  make rpm
#   Anything else, after `cargo build --release`:  make install
NAME    := whatsup-desktop
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TOP     := $(CURDIR)/target/rpm
SRC     := $(NAME)-$(VERSION)
SPEC    := packaging/fedora/$(NAME).spec
SPEC_VERSION := $(shell sed -n 's/^Version: *//p' $(SPEC))

.PHONY: rpm sources clean-rpm install uninstall

rpm: sources
	rpmbuild -ba --define "_topdir $(TOP)" $(SPEC)
	@find $(TOP)/RPMS -name '*.rpm' -print

# The source archive is the committed code (git archive), so a release's
# tarball matches its tag exactly; commit before building.
sources: clean-rpm
	@test "$(VERSION)" = "$(SPEC_VERSION)" || \
	    { echo "Cargo.toml ($(VERSION)) and $(SPEC) ($(SPEC_VERSION)) differ"; exit 1; }
	mkdir -p $(TOP)/SOURCES $(TOP)/vendor-tmp
	git archive --format=tar.gz --prefix=$(SRC)/ -o $(TOP)/SOURCES/$(SRC).tar.gz HEAD
	cargo vendor --locked --versioned-dirs $(TOP)/vendor-tmp/vendor > /dev/null
	tar -C $(TOP)/vendor-tmp -czf $(TOP)/SOURCES/$(SRC)-vendor.tar.gz vendor
	rm -rf $(TOP)/vendor-tmp

clean-rpm:
	rm -rf $(TOP)

# Installs a release build. Packagers stage it with DESTDIR=; PREFIX is where
# it lives on the system (/usr for packages, /usr/local by hand).
PREFIX  ?= /usr/local
BINARY  ?= target/release/$(NAME)
APP_ID  := app.whatsup_desktop.WhatsUp
SHARE   := $(DESTDIR)$(PREFIX)/share
SCALABLE_ICONS := $(APP_ID).svg $(APP_ID)-tray.svg $(APP_ID)-tray-unread.svg
SYMBOLIC_ICONS := $(APP_ID)-symbolic.svg $(APP_ID)-unread-symbolic.svg

install:
	install -Dm0755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/$(NAME)
	install -Dm0644 data/$(APP_ID).desktop $(SHARE)/applications/$(APP_ID).desktop
	install -Dm0644 data/$(APP_ID).metainfo.xml $(SHARE)/metainfo/$(APP_ID).metainfo.xml
	install -Dm0644 -t $(SHARE)/icons/hicolor/scalable/apps $(addprefix data/icons/,$(SCALABLE_ICONS))
	install -Dm0644 -t $(SHARE)/icons/hicolor/symbolic/apps $(addprefix data/icons/,$(SYMBOLIC_ICONS))

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(NAME) \
	    $(SHARE)/applications/$(APP_ID).desktop $(SHARE)/metainfo/$(APP_ID).metainfo.xml \
	    $(addprefix $(SHARE)/icons/hicolor/scalable/apps/,$(SCALABLE_ICONS)) \
	    $(addprefix $(SHARE)/icons/hicolor/symbolic/apps/,$(SYMBOLIC_ICONS))
