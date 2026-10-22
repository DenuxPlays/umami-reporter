FROM rust:1.99.1-alpine3.24 AS builder

RUN apk add --no-cache \
    musl-dev && \
    apk update

WORKDIR /usr/src/umami-reporter
COPY . .

RUN cargo build --release

# Final stage: create a minimal runtime image
FROM alpine:3.24.3

# Update package repositories
RUN apk update

# User creation
ARG UID=1000
ARG GUID=1000
ARG USER_NAME=umami-reporter
ENV GROUP_NAME=${USER_NAME}
ENV HOME_DIR=/home/${USER_NAME}

# Create a non-root user
RUN addgroup -g ${UID} -S ${GROUP_NAME} && \
    adduser -u ${GUID} -S -D -G ${GROUP_NAME} -h ${HOME_DIR} -s /bin/sh ${USER_NAME}

# Set the working directory
WORKDIR ${HOME_DIR}

# Copy the binaries from the builder stage
COPY --from=builder --chown=${UID}:${GUID} --chmod=+x /usr/src/umami-reporter/target/release/umami-reporter /home/umami-reporter/umami-reporter

# Switch to non-root user
USER ${USER_NAME}

ENTRYPOINT ["/home/umami-reporter/umami-reporter"]
