FROM ghcr.io/cross-rs/arm-unknown-linux-gnueabihf:edge

RUN curl -sL http://www.alsa-project.org/files/pub/lib/alsa-lib-1.2.11.tar.bz2 | tar -jxf -

RUN cd alsa-lib-1.2.11 && \
    AR=arm-unknown-linux-gnueabihf-ar CC=arm-unknown-linux-gnueabihf-gcc ./configure \
    --host=arm-unknown-linux-gnueabihf --enable-shared=yes --enable-static=no && \
    make -j4 && make install
