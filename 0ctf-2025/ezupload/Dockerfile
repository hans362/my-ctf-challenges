FROM dunglas/frankenphp:1.10.1-php8.4

ENV SERVER_NAME=:80

# The default Caddyfile from the frankenphp image
# Only add one line to remove the "Server: Caddy" header
COPY ./Caddyfile /etc/frankenphp/Caddyfile

# The default PHP production ini from the frankenphp image
# Only disable some classes and functions for the challenge
COPY ./php.ini "$PHP_INI_DIR/php.ini"

COPY ./index.php /app/public/index.php

RUN useradd app && \
  setcap CAP_NET_BIND_SERVICE=+eip /usr/local/bin/frankenphp && \
  chown -R app:app /config/caddy /data/caddy /app/public

COPY flag /flag
COPY readflag.c /readflag.c
RUN gcc -static -o /readflag /readflag.c && \
  rm /readflag.c && \
  chmod +s /readflag && \
  chmod 000 /flag

USER app