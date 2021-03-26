FROM nginx:1.19-alpine

RUN rm /etc/nginx/conf.d/default.conf
COPY docker/nginx/nginx.conf /etc/nginx/nginx.conf

ARG app_conf
COPY $app_conf /etc/nginx/conf.d/app.conf
