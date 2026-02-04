# Use official NGINX image as base
FROM nginx:alpine
RUN apk add --no-cache certbot certbot-nginx
COPY index.html /usr/share/nginx/html/
COPY robots.txt /usr/share/nginx/html/
COPY public /usr/share/nginx/html/public
COPY install /usr/share/nginx/html/install
COPY nginx.conf /etc/nginx/nginx.conf
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
EXPOSE 80
CMD ["/entrypoint.sh"]

