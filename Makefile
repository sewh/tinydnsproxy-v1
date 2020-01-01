.PHONY: build-server arm-debug arm-release x64-debug x64-release all-debug all-release \
	clean-builder-server clean-target clean x64-package arm-package all-package clean-package

build-server: .build-server-made

.build-server-made:
	docker build -t tinydnsproxy-build:latest .
	touch .build-server-made

all-release: arm-release x64-release
all-debug: arm-debug x64-debug
all-package: arm-package x64-package

arm-debug: target/armv7-unknown-linux-gnueabihf/debug/tinydnsproxy
arm-release: target/armv7-unknown-linux-gnueabihf/release/tinydnsproxy
arm-package: tinydnsproxy-armv7.tar.gz
tinydnsproxy-armv7.tar.gz: target/armv7-unknown-linux-gnueabihf/release/tinydnsproxy
	mkdir -p ./usr/local/bin
	cp -v target/armv7-unknown-linux-gnueabihf/release/tinydnsproxy usr/local/bin/tinydnsproxy
	tar cvzf tinydnsproxy-armv7.tar.gz usr
	rm -rf ./usr

target/armv7-unknown-linux-gnueabihf/debug/tinydnsproxy: build-server
	docker run -ti --rm -v $(shell pwd):/build tinydnsproxy-build /root/.cargo/bin/cargo build --target=armv7-unknown-linux-gnueabihf

target/armv7-unknown-linux-gnueabihf/release/tinydnsproxy: build-server
	docker run -ti --rm -v $(shell pwd):/build tinydnsproxy-build /root/.cargo/bin/cargo build --target=armv7-unknown-linux-gnueabihf --release

x64-debug: target/debug/tinydnsproxy
x64-release: target/release/tinydnsproxy
x64-package: tinydnsproxy-x64.tar.gz
tinydnsproxy-x64.tar.gz: target/release/tinydnsproxy
	mkdir -p ./usr/local/bin
	cp -v target/release/tinydnsproxy ./usr/local/bin/tinydnsproxy
	tar cvzf tinydnsproxy-x64.tar.gz usr
	rm -rf ./usr

target/debug/tinydnsproxy: build-server
	docker run -ti --rm -v $(shell pwd):/build tinydnsproxy-build /root/.cargo/bin/cargo build

target/release/tinydnsproxy: build-server
	docker run -ti --rm -v $(shell pwd):/build tinydnsproxy-build /root/.cargo/bin/cargo build --release

clean-build-server:
	docker rmi -f tinydnsproxy-build:latest
	rm -fv .build-server-made

clean-target:
	rm -rfv target

clean-package:
	rm -fv tinydnsproxy-x64.tar.gz tinydnsproxy-armv7.tar.gz

clean: clean-build-server clean-target
