#!/bin/bash
# generate_certs.sh - Generate self-signed TLS certificates for local dev/testing
set -euo pipefail

CERT_DIR="${1:-./nginx/ssl}"
mkdir -p "$CERT_DIR"

echo "[*] Generating self-signed TLS certificates in $CERT_DIR"

openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout "$CERT_DIR/key.pem" \
  -out "$CERT_DIR/cert.pem" \
  -subj "/C=US/ST=State/L=City/O=APEX/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

chmod 600 "$CERT_DIR/key.pem"
chmod 644 "$CERT_DIR/cert.pem"

echo "[+] Certificates generated:"
echo "    Key:   $CERT_DIR/key.pem"
echo "    Cert:  $CERT_DIR/cert.pem"
echo "[*] NOTE: For production, use certificates from a trusted CA (Let's Encrypt, internal CA, etc.)"
echo "[*] NOTE: Add *.pem files to .gitignore to avoid accidentally committing secrets"