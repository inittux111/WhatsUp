# Builds the RPM in the whatsup-desktop toolbox: make rpm
NAME    := whatsup-desktop
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TOP     := $(CURDIR)/target/rpm
SRC     := $(NAME)-$(VERSION)
SPEC_VERSION := $(shell sed -n 's/^Version: *//p' $(NAME).spec)

.PHONY: rpm sources clean-rpm

rpm: sources
	rpmbuild -ba --define "_topdir $(TOP)" $(NAME).spec
	@find $(TOP)/RPMS -name '*.rpm' -print

# The source archive is the committed code (git archive), so a release's
# tarball matches its tag exactly; commit before building.
sources: clean-rpm
	@test "$(VERSION)" = "$(SPEC_VERSION)" || \
	    { echo "Cargo.toml ($(VERSION)) and $(NAME).spec ($(SPEC_VERSION)) differ"; exit 1; }
	mkdir -p $(TOP)/SOURCES $(TOP)/vendor-tmp
	git archive --format=tar.gz --prefix=$(SRC)/ -o $(TOP)/SOURCES/$(SRC).tar.gz HEAD
	cargo vendor --locked --versioned-dirs $(TOP)/vendor-tmp/vendor > /dev/null
	tar -C $(TOP)/vendor-tmp -czf $(TOP)/SOURCES/$(SRC)-vendor.tar.gz vendor
	rm -rf $(TOP)/vendor-tmp

clean-rpm:
	rm -rf $(TOP)
