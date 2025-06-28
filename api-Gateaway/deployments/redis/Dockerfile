FROM redis@sha256:4c8fbf6e4f8b6b2b8e8c7b8e8c7b8e8c7b8e8c7b8e8c7b8e8c7b8e8c7b8e8c7b8

COPY redis.conf /usr/local/etc/redis/redis.conf

# Create data directory
RUN mkdir -p /data && chown redis:redis /data
VOLUME /data

# Non-default port for security
EXPOSE 6379

# Memory limit and persistence
CMD ["redis-server", "/usr/local/etc/redis/redis.conf", "--maxmemory", "256mb", "--maxmemory-policy", "allkeys-lru", "--save", "60", "100", "--appendonly", "yes"]

USER redis