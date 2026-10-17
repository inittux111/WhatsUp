# Builds the RPM in the whatsup-desktop toolbox: make rpm
NAME    := whatsup-desktop
VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TOP     := $(CURDIR)/target/rpm
SRC     := $(NAME)-$(VERSION)

.PHONY: rpm sources clean-rpm

rpm: sources
	rpmbuild -ba --define "_topdir $(TOP)" $(NAME).spec
	@find $(TOP)/RPMS -name '*.rpm' -print

sources: clean-rpm
	mkdir -p $(TOP)/SOURCES $(TOP)/vendor-tmp
	tar --exclude=./target --exclude=./.git --exclude=./vendor --exclude=./.cargo \
	    --transform 's,^\.,$(SRC),' -czf $(TOP)/SOURCES/$(SRC).tar.gz .
	cargo vendor --locked --versioned-dirs $(TOP)/vendor-tmp/vendor > /dev/null
	tar -C $(TOP)/vendor-tmp -czf $(TOP)/SOURCES/$(SRC)-vendor.tar.gz vendor
	rm -rf $(TOP)/vendor-tmp

clean-rpm:
	rm -rf $(TOP)
