#!/bin/bash
# Install simple-http-server if not present
if ! command -v simple-http-server &> /dev/null; then
    echo "Installing simple-http-server..."
    cargo install simple-http-server
fi

echo "Serving at http://localhost:3000"
simple-http-server -i ./www/ -p 3000 --nocache --try-file ./www/index.html
