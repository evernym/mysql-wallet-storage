FROM evernym/rust:0.2.0-centos7
# TODO LABEL maintainer="Name <email-address>"

ARG u_id=1000

ARG u_name=user

# project's specific dependencies
#RUN yum install -y \
#        ... \
#    && yum clean all

RUN useradd -ms /bin/bash -u $u_id $u_name
USER $u_name

# TODO CMD ENTRYPOINT ...
WORKDIR /home/$u_name

ENV CI_ENV_VERSION=0.2.0
