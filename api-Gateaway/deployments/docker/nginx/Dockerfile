FROM nginx:1.23-alpine

# Remove default configs
RUN rm /etc/nginx/conf.d/default.conf

# Copy custom configuration
COPY nginx.conf /etc/nginx/nginx.conf
COPY sites-enabled/ /etc/nginx/sites-enabled/

# Install Lua for dynamic configs
RUN apk add --no-cache lua5.3

# SSL directories
RUN mkdir -p /etc/nginx/ssl
RUN chmod 700 /etc/nginx/ssl

# Health check
HEALTHCHECK --interval=10s --timeout=2s \
    CMD curl -f http://localhost/health || exit 1

EXPOSE 80 443

VOLUME ["/etc/nginx/ssl", "/var/log/nginx"]