# AWS EC2 Deployment Guide - GOLD-e Marketing API (Rust)

This guide walks through deploying the Rust-based Marketing API to an AWS EC2 instance (Ubuntu 22.04 LTS / Debian 12).

---

## 1. Instance Sizing & Security Group Rules

- **Instance Type:** `t3.small` or `t3.medium` (2 vCPU, 2-4 GB RAM)
- **Security Group Inbound Rules:**
  - `SSH` (Port 22) -> Your Admin IP
  - `HTTP` (Port 80) -> 0.0.0.0/0
  - `HTTPS` (Port 443) -> 0.0.0.0/0
  - `PostgreSQL` (Port 5432) -> Internal VPC only (do not expose publicly)

---

## 2. Automated Provisioning

1. SSH into your EC2 instance:
```bash
ssh -i your-key.pem ubuntu@<ec2-public-ip>
```

2. Clone or transfer the `deploy/` directory to the server:
```bash
scp -i your-key.pem -r deploy/ ubuntu@<ec2-public-ip>:/tmp/deploy/
```

3. Run the provisioning script:
```bash
chmod +x /tmp/deploy/aws-ec2-setup.sh
sudo /tmp/deploy/aws-ec2-setup.sh
```

---

## 3. Continuous Delivery via GitHub Actions

Configure the following GitHub Secrets in your repository:
- `EC2_HOST`: Elastic IP of your EC2 instance
- `EC2_USERNAME`: `ubuntu`
- `EC2_SSH_PRIVATE_KEY`: Your private SSH key (`.pem`)

On push to `main`, GitHub Actions automatically compiles the release binary, scps it to `/opt/golde-marketing-api/bin/`, and restarts the systemd service.

---

## 4. Manual Service Management

```bash
# Check service status
sudo systemctl status golde-marketing-api

# View live application logs
journalctl -u golde-marketing-api -f

# Restart application
sudo systemctl restart golde-marketing-api
```
