# The HTTP service of a-graph: one container answers `POST /run` with the graph of the caller.
#
#   docker build -t a-graph .
#   docker run -p 8080:8080 \
#     -v "$PWD/service.yml:/etc/a-graph/service.yml:ro" \
#     -v a-graph-data:/data \
#     -e ANTHROPIC_API_KEY -e OPENAI_API_KEY \
#     a-graph
#
# The keys are variables of the container: name here the ones the graphs use, their values come
# from your shell (or `--env-file`). There is nothing in the image but the program.

# the minor is pinned, the patch is not: a patch of the toolchain is a fix, never a change of the language
FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
# the dependencies alone first, on an empty crate: their layer is kept until Cargo.lock changes, here and in the
# cache of the CI, so after a change in src/ only the crate itself is compiled again
RUN mkdir src && echo 'fn main() {}' > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src
COPY src ./src
# the sources are older than the empty crate built above: touched, so that cargo builds them
RUN touch src/main.rs src/lib.rs \
    && cargo build --release --locked \
    && cp target/release/a-graph /usr/local/bin/a-graph

FROM debian:bookworm-slim
# the certificates are the ones the program needs to talk https; a pinned version of the package would
# stop building when Debian publishes a security update, so it is not pinned (hadolint DL3008)
# hadolint ignore=DL3008
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home /data a-graph \
    && mkdir /data && chown 10001 /data
COPY --from=build /usr/local/bin/a-graph /usr/local/bin/a-graph
# a number, so that an orchestrator can check that the container does not run as root
USER 10001
VOLUME /data
EXPOSE 8080
ENTRYPOINT ["a-graph"]
CMD ["--serve", "/etc/a-graph/service.yml"]
