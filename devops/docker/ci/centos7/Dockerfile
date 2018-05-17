FROM evernym/rust:centos7
# TODO LABEL maintainer="Name <email-address>"

ARG u_id=1000
ARG u_name=user

ENV CI_ENV_VERSION=0.3.0

# - install recent libsodium version from the sources
#  recent libsodium is available in EPEL
# - install sql as well
RUN wget -P /tmp http://dl.fedoraproject.org/pub/epel/epel-release-latest-7.noarch.rpm \
        && rpm -ivh /tmp/epel-release-latest-7.noarch.rpm \
        && rm -f /tmp/epel-release-latest-7.noarch.rpm \
    && yum install -y \
        libsodium-devel \
        sqlite-devel \
    && yum clean all

RUN useradd -ms /bin/bash -u $u_id $u_name
USER $u_name

# TODO CMD ENTRYPOINT ...
WORKDIR /home/$u_name
