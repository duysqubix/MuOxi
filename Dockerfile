FROM rust:1.47.0-slim AS runtime
ARG MUOXI_INSTALL_DIR=/opt/muoxi
ENV LANG=C.UTF-8 MUOXI_INSTALL_DIR=${MUOXI_INSTALL_DIR}
WORKDIR ${MUOXI_INSTALL_DIR}
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    apt-transport-https software-properties-common \
    ca-certificates \
    libpq5 \
    openssl \
    curl \
    tzdata && \
    rm -rf /var/lib/apt/lists/*
RUN export DOCKERIZE_VERSION=v0.6.1 && curl -L \
    https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    | tar -C /usr/local/bin -xz

    
FROM runtime AS builder
ARG MUOXI_UID=1000
ARG MUOXI_USERNAME=you
ENV MUOXI_UID=${MUOXI_UID}
COPY . ${MUOXI_INSTALL_DIR}
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        git \
        libpq-dev && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -M -u ${MUOXI_UID} -d ${MUOXI_INSTALL_DIR} -c "MuOxi user,,," ${MUOXI_USERNAME} && \
    chown -R $MUOXI_UID ${MUOXI_INSTALL_DIR}
USER ${MUOXI_UID}
RUN \
    cargo install diesel_cli --no-default-features --features postgres && \
    cargo install --path=${MUOXI_INSTALL_DIR}/muoxi
CMD [ "cargo", "run", "--bin", "muoxi_staging" ]

FROM runtime AS release
COPY --from=builder /usr/local/bundle /usr/local/bundle
COPY --from=builder --chown=nobody:nogroup ${MUOXI_INSTALL_DIR} ${MUOXI_INSTALL_DIR}
USER nobody
CMD [ "cargo", "run", "--bin", " muoxi_web" ]
ARG SOURCE_BRANCH="master"
ARG SOURCE_COMMIT="000000"
ARG BUILD_DATE="2020-10-11T12:53:26Z"
ARG IMAGE_NAME="MuOxi:latest"

LABEL org.label-schema.build-date=$BUILD_DATE \
      org.label-schema.name="MuOxi" \
      org.label-schema.description="MuOxi" \
      org.label-schema.vcs-url="https://github.com/duysqubix/MuOxi.git" \
      org.label-schema.vcs-ref=$SOURCE_COMMIT \
      org.label-schema.schema-version="1.0.0-rc1" \
      build-target="release" \
      build-branch=$SOURCE_BRANCH

