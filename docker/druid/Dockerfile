# Note this image requires at least 4 Gigs of RAM
FROM openjdk:8-jre-alpine

RUN set -ex; apk add --no-cache bash perl python curl

RUN set -ex; \
    curl -fsLS "http://mirror.cc.columbia.edu/pub/software/apache/druid/0.19.0/apache-druid-0.19.0-bin.tar.gz" | gunzip | tar x -C /opt; \
    mv /opt/apache-druid-0.19.0 /opt/druid

WORKDIR /opt/druid/

ADD config/common.runtime.properties conf/druid/single-server/micro-quickstart/_common/common.runtime.properties
ADD config/countries-index.json quickstart/tutorial/countries-index.json 
ADD config/countries.csv quickstart/tutorial/countries.csv

RUN set -ex; \
    bash -c "./bin/start-micro-quickstart &"; \
    ./bin/post-index-task --file quickstart/tutorial/wikipedia-index.json --url http://localhost:8081 --submit-timeout 600; \
    ./bin/post-index-task --file quickstart/tutorial/countries-index.json --url http://localhost:8081 --submit-timeout 600

# router
EXPOSE 8888 
# coordinator
EXPOSE 8081
# broker
EXPOSE 8082
# historical
EXPOSE 8083
# overlord
EXPOSE 8090
# ZooKeeper
EXPOSE 2181 2888 3888

ENTRYPOINT ./bin/start-micro-quickstart
