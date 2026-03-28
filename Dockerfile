# Build stage
FROM rust:alpine AS builder

RUN apk add --no-cache \
    musl-dev \
    build-base \
    pkgconf \
    sqlite-dev \
    sqlite-static \
    nodejs \
    npm \
    && npm install -g pnpm

WORKDIR /app

# Install JS deps first for better layer caching
COPY templates/package.json templates/pnpm-lock.yaml ./templates/
RUN cd templates && pnpm install --frozen-lockfile

# Copy source (node_modules excluded via .dockerignore)
COPY Cargo.toml Cargo.lock ./
COPY src        ./src
COPY migrations ./migrations
COPY templates  ./templates

RUN cargo build --release

# Runtime stage
FROM alpine:3.23

RUN apk add --no-cache sqlite-libs ca-certificates tzdata

# Create a non-root system user with no home dir and no login shell
RUN addgroup -S laneya \
 && adduser -S -G laneya -H -s /sbin/nologin laneya

WORKDIR /app

COPY --from=builder --chown=laneya:laneya /app/target/release/laneya      ./
COPY --from=builder --chown=laneya:laneya /app/templates/static           ./templates/static

# Create the data directory with correct ownership BEFORE declaring VOLUME
# so anonymous/named volumes inherit this ownership on first mount
RUN mkdir -p /data && chown laneya:laneya /data

VOLUME ["/data"]

EXPOSE 8080

ENV DATABASE_URL=/data/database.db
ENV HOST=0.0.0.0:8080
ENV TZ=UTC

USER laneya

CMD ["./laneya"]

