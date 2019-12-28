FROM ubuntu:latest

RUN apt-get update && apt-get upgrade -y && apt-get -y install cron openssh-client
RUN useradd --home-dir /var/lib/speculo --create-home speculo --system
COPY ./target/release/speculo /usr/local/bin

COPY ./scripts/push-all.sh /usr/local/bin
COPY ./scripts/create-keys.sh /usr/local/bin
RUN chmod +x /usr/local/bin/push-all.sh
RUN chmod +x /usr/local/bin/create-keys.sh

COPY ./scripts/speculo-run /etc/cron.d/speculo-run
RUN chmod 0644 /etc/cron.d/speculo-run

CMD ["/usr/local/bin/push-all.sh"]
