FROM evernym/rust:0.2.0-xenial
# TODO LABEL maintainer="Name <email-address>"

ARG u_id=1000
ARG u_name=user

# project's specific dependencies
#RUN apt-get update && apt-get install -y --no-install-recommends \
#        ... \
#	&& rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash -u $u_id $u_name
USER $u_name

# TODO CMD ENTRYPOINT ...
WORKDIR /home/$u_name

ENV CI_ENV_VERSION=0.2.0
