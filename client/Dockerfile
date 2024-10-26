FROM rust:1.80 as builder
LABEL authors="jessegreenough"
WORKDIR ./

COPY . .

RUN rustup target add wasm32-unknown-unknown
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install --locked trunk
RUN SERVER_URL=/api trunk build --release


FROM nginx:alpine
COPY --from=builder /dist /usr/share/nginx/html
COPY ./nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

# Start Nginx server
CMD ["nginx", "-g", "daemon off;"]