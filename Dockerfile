FROM debian:10

RUN dpkg --add-architecture armhf && apt-get update && apt-get install -y curl gcc pkg-config openssl libssl-dev libssl-dev:armhf gcc-8-arm-linux-gnueabihf && ln /usr/bin/arm-linux-gnueabihf-gcc-8 /usr/bin/arm-linux-gnueabihf-gcc
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > /tmp/rustup.sh && chmod +x /tmp/rustup.sh && sh /tmp/rustup.sh -y && echo 'source /root/.cargo/env' >> /root/.bashrc
RUN /root/.cargo/bin/rustup target add armv7-unknown-linux-gnueabihf
ENV PKG_CONFIG_ALLOW_CROSS=1
