#!/usr/bin/env bash
set -euo pipefail

echo "========================================================="
echo " GOLD-e GrowthOS Marketing API - AWS EC2 Provisioning "
echo "========================================================="

# 1. Update OS Packages
echo " Updating system packages..."
sudo apt-get update -y && sudo apt-get upgrade -y
sudo apt-get install -y curl wget git build-essential pkg-config libssl-dev postgresql postgresql-contrib nginx ufw

# 2. Create System User and Directory Structure
echo " Creating system application directories..."
sudo useradd -r -s /bin/false golde || true
sudo mkdir -p /opt/golde-marketing-api/bin
sudo mkdir -p /opt/golde-marketing-api/migrations
sudo mkdir -p /var/log/golde-marketing-api
sudo chown -R golde:golde /opt/golde-marketing-api /var/log/golde-marketing-api

# 3. Setup PostgreSQL
echo " Configuring PostgreSQL database and user..."
sudo -u postgres psql -c "CREATE USER golde WITH PASSWORD 'SecureProductionDBPass2026!';" || true
sudo -u postgres psql -c "CREATE DATABASE marketing_api OWNER golde;" || true
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE marketing_api TO golde;" || true

# 4. Install Systemd Service
echo " Installing systemd service..."
sudo cp /tmp/golde-marketing-api.service /etc/systemd/system/golde-marketing-api.service
sudo systemctl daemon-reload
sudo systemctl enable golde-marketing-api

# 5. Configure Nginx Reverse Proxy & TLS
echo " Configuring Nginx reverse proxy..."
sudo cp /tmp/nginx.conf /etc/nginx/sites-available/golde-marketing-api
sudo ln -sf /etc/nginx/sites-available/golde-marketing-api /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t
sudo systemctl restart nginx

# 6. Configure Firewall
echo "🛡️ Configuring UFW firewall rules..."
sudo ufw allow OpenSSH
sudo ufw allow 'Nginx Full'
sudo ufw --force enable

echo "✅ Provisioning complete. Copy your release binary to /opt/golde-marketing-api/bin/golde-marketing-api and start the service."
