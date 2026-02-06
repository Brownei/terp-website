# Use official NGINX image as base
FROM nginx:alpine
RUN apk add --no-cache certbot certbot-nginx
COPY index.html /usr/share/nginx/html/
COPY robots.txt /usr/share/nginx/html/
COPY public /usr/share/nginx/html/public
COPY get /usr/share/nginx/html/get
COPY docker/nginx.conf /etc/nginx/nginx.conf
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
EXPOSE 80
CMD ["/entrypoint.sh"]

