FROM rust:bullseye
RUN \
  apt-get update && \
  apt-get install git -y && \
  mkdir ~/.ssh/ && \
  ssh-keyscan -t rsa github.com > ~/.ssh/known_hosts
WORKDIR /rocket-sqlx-authentication-api
COPY . .
RUN ls /rocket-sqlx-authentication-api
RUN cd /rocket-sqlx-authentication-api
RUN cargo build
CMD ["target/debug/rocket-sqlx-authentication-api"]
EXPOSE 8030
