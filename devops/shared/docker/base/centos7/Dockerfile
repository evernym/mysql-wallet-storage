FROM centos:7
# TODO LABEL maintainer="Name <email-address>"

# generally useful packages
# TODO not sure that we need yum update
RUN yum install -y \
        curl \
        wget \
        vim \
        git \
        procps \
		autoconf \
		automake \
		g++ \
		gcc \
		make \
    && yum clean all

# install fpm
ENV FPM_VERSION=1.9.3
RUN yum install -y \
        rpm-build \
        ruby \
        ruby-devel \
        rubygems \
    && gem install --no-ri --no-rdoc fpm -v $FPM_VERSION \
    && yum clean all

# TODO CMD ENTRYPOINT ...
