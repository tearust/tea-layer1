FROM ubuntu:20.10

COPY ./tmp/tea-layer1 /usr/local/bin/

EXPOSE 9944
EXPOSE 9933

CMD ["tea-layer1", "--dev", "--ws-external"]
