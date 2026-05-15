FROM rust:1.85-slim AS builder
ARG MUOXI_INSTALL_DIR=/opt/muoxi
ENV LANG=C.UTF-8 MUOXI_INSTALL_DIR=${MUOXI_INSTALL_DIR}
WORKDIR ${MUOXI_INSTALL_DIR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY . ${MUOXI_INSTALL_DIR}
RUN cargo build --release --workspace

FROM debian:bookworm-slim AS runtime
ARG MUOXI_INSTALL_DIR=/opt/muoxi
ENV MUOXI_INSTALL_DIR=${MUOXI_INSTALL_DIR}
WORKDIR ${MUOXI_INSTALL_DIR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -r -M -d ${MUOXI_INSTALL_DIR} muoxi

COPY --from=builder ${MUOXI_INSTALL_DIR}/target/release/muoxi_server /usr/local/bin/muoxi_server
COPY --from=builder ${MUOXI_INSTALL_DIR}/target/release/muoxi_web /usr/local/bin/muoxi_web
COPY --from=builder ${MUOXI_INSTALL_DIR}/migrations ${MUOXI_INSTALL_DIR}/migrations
COPY --from=builder ${MUOXI_INSTALL_DIR}/resources ${MUOXI_INSTALL_DIR}/resources

# chown must precede USER so named-volume permissions propagate on first mount.
RUN mkdir -p ${MUOXI_INSTALL_DIR}/data && chown -R muoxi:muoxi ${MUOXI_INSTALL_DIR}

USER muoxi
EXPOSE 8000 8080
CMD ["muoxi_server"]

ARG SOURCE_BRANCH="master"
ARG SOURCE_COMMIT="000000"
ARG BUILD_DATE="2026-05-07T00:00:00Z"
ARG IMAGE_NAME="MuOxi:latest"

LABEL org.label-schema.build-date=$BUILD_DATE \
      org.label-schema.name="MuOxi" \
      org.label-schema.description="MuOxi MUD framework" \
      org.label-schema.vcs-url="https://github.com/duysqubix/MuOxi.git" \
      org.label-schema.vcs-ref=$SOURCE_COMMIT \
      org.label-schema.schema-version="1.0.0-rc1" \
      build-target="release" \
      build-branch=$SOURCE_BRANCH
