#!/bin/bash
# Install simple-http-server if not present
if ! command -v simple-http-server &> /dev/null; then
    echo "Installing simple-http-server..."
    cargo install simple-http-server
fi

# Kill any process running on port 3000
if lsof -ti:3000 &> /dev/null; then
    echo "Killing process on port 3000..."
    lsof -ti:3000 | xargs kill -9
fi

DIST_DIR="./www/dist"

if [ ! -d "$DIST_DIR" ]; then
    echo "Build directory $DIST_DIR not found. Please run 'cd www && npm run build' first."
    exit 1
fi

# Open browser after a slight delay
if command -v open &> /dev/null; then
    (sleep 1 && open "http://localhost:3000") &
elif command -v xdg-open &> /dev/null; then
    (sleep 1 && xdg-open "http://localhost:3000") &
fi

echo "Serving React App via rxq-server at http://localhost:3000"
# Run the Rust backend server
cargo run -p rxq-server
