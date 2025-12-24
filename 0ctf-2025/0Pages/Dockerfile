FROM rust:slim

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
  echo 'fn main() { println!("Placeholder"); }' > src/main.rs && \
  cargo build --release

COPY src ./src/
RUN touch src/main.rs && \
  cargo build --release

RUN apt-get update && \
  apt-get install -y --no-install-recommends ca-certificates apache2 sudo zip && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*
COPY apache2.conf /etc/apache2/apache2.conf
COPY 000-default.conf /etc/apache2/sites-available/000-default.conf
RUN a2enmod proxy proxy_http rewrite
RUN a2dismod status
COPY static/. /var/www/html/

COPY readflag.c /readflag.c
COPY flag /flag
RUN gcc -static -o /readflag /readflag.c && \
  rm /readflag.c && \
  chmod +s /readflag && \
  chmod 000 /flag

RUN chown -R www-data:www-data /var/www/html && \
  chown -R www-data:www-data /app

COPY .secretkey /app/.secretkey
RUN chmod 444 /app/.secretkey

CMD ["/bin/bash", "-c", "apache2ctl -D FOREGROUND & sudo -u www-data ./target/release/zero-pages"]