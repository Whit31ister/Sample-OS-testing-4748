FROM rust:1.94-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        binutils \
        grub-common \
        grub-pc-bin \
        mtools \
        qemu-system-x86 \
        xorriso \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add i686-unknown-linux-gnu

WORKDIR /workspace

CMD ["bash", "-lc", "./my_os/scripts/build.sh"]
