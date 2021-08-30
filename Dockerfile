FROM rust:1.54 as builder
WORKDIR /usr/src/r0le
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
ENV TOKEN=
ENV APPLICATION_ID=
ENV REDIS_URL=
COPY --from=builder /usr/local/cargo/bin/r0le /usr/local/bin/r0le
RUN rm -rf /var/lib/apt/lists/*
CMD ["sh", "-c", "r0le -t ${TOKEN} -a ${APPLICATION_ID} -r ${REDIS_URL}"]