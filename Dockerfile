FROM rust:1.47.0-slim AS runtime
ENV LANG C.UTF-8
WORKDIR /usr/src
ENV HOME=/usr/src PATH=/usr/src/bin:$PATH
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

    
FROM runtime AS development
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    git \
    libpq-dev && \
    rm -rf /var/lib/apt/lists/*
COPY . /usr/src/
RUN cargo install --path=/usr/src/muoxi 
CMD [ "cargo", "run", "--bin", "muoxi_staging" ]

FROM development AS testing
COPY . /usr/src
ARG DEVELOPER_UID=1000
ARG DEVELOPER_USERNAME=you
ENV DEVELOPER_UID=${DEVELOPER_UID}
RUN useradd -r -M -u ${DEVELOPER_UID} -d /usr/src -c "Developer User,,," ${DEVELOPER_USERNAME} \
 && echo ${DEVELOPER_USERNAME} ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/${DEVELOPER_USERNAME} \
 && chmod 0440 /etc/sudoers.d/${DEVELOPER_USERNAME}


FROM testing AS builder
RUN cargo install diesel_cli --no-default-features --postgres
RUN export DATABASE_URL=postgres://postgres@example.com:5432/fakedb \
    diesel migration run


FROM runtime AS release
COPY --from=builder /usr/local/bundle /usr/local/bundle
COPY --from=builder --chown=nobody:nogroup /usr/src /usr/src
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

