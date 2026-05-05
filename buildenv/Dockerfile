FROM randomdude/gcc-cross-x86_64-elf

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        grub-common \
        grub-pc-bin \
        make \
        nasm \
        xorriso \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
VOLUME ["/workspace"]

CMD ["/bin/bash"]
