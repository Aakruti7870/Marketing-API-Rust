#!/usr/bin/env bash
set -euo pipefail

APP_DIR=/opt/golde-marketing-api
RELEASE_DIR=/tmp/golde-marketing-api-release
SERVICE_FILE=$APP_DIR/golde-marketing-api.service
NGINX_FILE=/etc/nginx/sites-available/golde-marketing-api

sudo apt-get update -y
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y postgresql postgresql-contrib nginx curl certbot python3-certbot-nginx

if ! id golde >/dev/null 2>&1; then
  sudo useradd --system --home "$APP_DIR" --shell /usr/sbin/nologin golde
fi

sudo mkdir -p "$APP_DIR/bin" "$APP_DIR/migrations" /var/log/golde-marketing-api
BINARY_PATH=$(find "$RELEASE_DIR" -type f -name golde-marketing-api -print -quit)
if [ -z "$BINARY_PATH" ]; then
  echo "Release binary not found under $RELEASE_DIR" >&2
  exit 1
fi
sudo cp "$BINARY_PATH" "$APP_DIR/bin/golde-marketing-api"
find "$RELEASE_DIR" -type f -name '*.sql' -exec sudo cp {} "$APP_DIR/migrations/" \;
sudo chmod 755 "$APP_DIR/bin/golde-marketing-api"

sudo systemctl enable --now postgresql

DB_PASS_FILE="$APP_DIR/.db_password"
if [ ! -s "$DB_PASS_FILE" ]; then
  sudo openssl rand -hex 32 | sudo tee "$DB_PASS_FILE" >/dev/null
  sudo chmod 600 "$DB_PASS_FILE"
fi
DB_PASS=$(sudo cat "$DB_PASS_FILE")

sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='golde'" | grep -q 1 || sudo -u postgres psql -c "CREATE ROLE golde LOGIN PASSWORD '$DB_PASS';"
sudo -u postgres psql -c "ALTER ROLE golde WITH PASSWORD '$DB_PASS';"
sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='marketing_api'" | grep -q 1 || sudo -u postgres createdb -O golde marketing_api
sudo -u postgres psql -c "ALTER DATABASE marketing_api OWNER TO golde;"

ACCESS_FILE="$APP_DIR/.jwt_access_secret"
REFRESH_FILE="$APP_DIR/.jwt_refresh_secret"
if [ ! -s "$ACCESS_FILE" ]; then sudo openssl rand -hex 48 | sudo tee "$ACCESS_FILE" >/dev/null; sudo chmod 600 "$ACCESS_FILE"; fi
if [ ! -s "$REFRESH_FILE" ]; then sudo openssl rand -hex 48 | sudo tee "$REFRESH_FILE" >/dev/null; sudo chmod 600 "$REFRESH_FILE"; fi

sudo tee "$APP_DIR/.env" >/dev/null <<EOF
PORT=4000
HOST=127.0.0.1
ENVIRONMENT=production
DATABASE_URL=postgresql://golde:$DB_PASS@localhost:5432/marketing_api?sslmode=disable
JWT_ACCESS_SECRET=$(sudo cat "$ACCESS_FILE")
JWT_REFRESH_SECRET=$(sudo cat "$REFRESH_FILE")
JWT_ACCESS_EXPIRATION_SECONDS=900
JWT_REFRESH_EXPIRATION_SECONDS=604800
CORS_ORIGIN=*
RATE_LIMIT_REQUESTS_PER_MINUTE=120
WHATSAPP_SIMULATION_MODE=true
RUST_LOG=info,golde_marketing_api=info,tower_http=info
EOF
sudo chmod 600 "$APP_DIR/.env"

sudo tee "$SERVICE_FILE" >/dev/null <<'EOF'
[Unit]
Description=GOLD-e GrowthOS Marketing API Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=golde
Group=golde
WorkingDirectory=/opt/golde-marketing-api
EnvironmentFile=/opt/golde-marketing-api/.env
ExecStart=/opt/golde-marketing-api/bin/golde-marketing-api
Restart=always
RestartSec=5s
LimitNOFILE=65535
ProtectSystem=full
ProtectHome=true
NoNewPrivileges=true
PrivateTmp=true
StandardOutput=append:/var/log/golde-marketing-api/output.log
StandardError=append:/var/log/golde-marketing-api/error.log

[Install]
WantedBy=multi-user.target
EOF

sudo chown -R golde:golde "$APP_DIR" /var/log/golde-marketing-api
sudo chmod 700 "$APP_DIR"
sudo systemctl daemon-reload
sudo systemctl enable --now golde-marketing-api

sudo tee "$NGINX_FILE" >/dev/null <<'EOF'
upstream golde_backend {
    server 127.0.0.1:4000;
    keepalive 64;
}

server {
    listen 80;
    server_name api.goldetech.com;

    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    location / {
        proxy_pass http://golde_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 60s;
    }
}
EOF

sudo ln -sfn "$NGINX_FILE" /etc/nginx/sites-enabled/golde-marketing-api
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl reload nginx

sleep 3
curl -fsS http://127.0.0.1:4000/health
curl -fsS -H 'Host: api.goldetech.com' http://127.0.0.1/health

sudo ufw allow OpenSSH || true
sudo ufw allow 'Nginx Full' || true
sudo ufw --force enable || true

if ! sudo certbot certificates 2>/dev/null | grep -q 'api.goldetech.com'; then
  sudo certbot --nginx --non-interactive --agree-tos --register-unsafely-without-email --redirect -d api.goldetech.com
fi

curl -fsS https://api.goldetech.com/health
