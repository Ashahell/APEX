FROM node:24-alpine AS builder

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .
RUN pnpm build

FROM nginx:alpine

# Create non-root user (nginx already runs as nginx user, but be explicit)
RUN addgroup -g 1001 -S apex-ui && adduser -S -u 1001 -G apex-ui apex-ui

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Set proper permissions
RUN chown -R apex-ui:apex-ui /usr/share/nginx/html /var/cache/nginx /var/log/nginx /etc/nginx/conf.d && \
    chmod -R 755 /usr/share/nginx/html && \
    touch /var/run/nginx.pid && \
    chown apex-ui:apex-ui /var/run/nginx.pid

# Switch to non-root user
USER apex-ui

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
