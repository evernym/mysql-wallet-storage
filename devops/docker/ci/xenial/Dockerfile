FROM evernym/rust:xenial
# TODO LABEL maintainer="Name <email-address>"

ARG u_id=1000
ARG u_name=user

ENV CI_ENV_VERSION=0.1.0

# project's specific dependencies
#RUN apt-get update && apt-get install -y --no-install-recommends \
#        libsqlite3-dev \
#	&& rm -rf /var/lib/apt/lists/*

# install recent libsodium version from the sources
#ENV LIBSODIUM_VERSION=1.0.16
#RUN cd /tmp \
#    && curl https://download.libsodium.org/libsodium/releases/libsodium-${LIBSODIUM_VERSION}.tar.gz | tar -xz \
#    && cd /tmp/libsodium-${LIBSODIUM_VERSION} \
#    && ./configure --prefix=/usr/local/ && make && make install \
#    && ldconfig \
#    && rm -rf /tmp/libsodium-${LIBSODIUM_VERSION}
# need for libsodium to be reachable via pkg-config (sodiumoxide uses it)
#ENV PKG_CONFIG_PATH /usr/local/lib/pkgconfig:$PKG_CONFIG_PATH # TODO ??? is it really needed

RUN useradd -ms /bin/bash -u $u_id $u_name
USER $u_name

# TODO CMD ENTRYPOINT ...
WORKDIR /home/$u_name
