#!/bin/bash

# Setup git repository for Chiral Network

echo "Setting up Chiral Network git repository..."

# Navigate to the project directory
cd /Users/shuai/workspace/brooknet/chiral-network/chiral-app

# Initialize git repository
git init

# Configure git (you may want to update these)
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Add all files
git add .

# Create initial commit
git commit -m "Initial commit: Chiral Network - Decentralized P2P File Sharing Platform

This is the initial commit for Chiral Network, a decentralized peer-to-peer 
file storage and sharing system built with:

- Frontend: Svelte + TypeScript + Tailwind CSS
- Desktop: Tauri framework
- Blockchain: Ethereum-compatible network (Chain ID: 98765)
- Storage: DHT-based distributed file storage
- Features: P2P file sharing, mining, analytics, and more

Key components:
- BitTorrent-style continuous seeding model
- No marketplace features (non-commercial focus)
- Privacy-first architecture with proxy support
- Comprehensive documentation in /docs

🚀 Ready to build the decentralized future of file sharing!"

# Add remote repository
git remote add origin git@github.com:chiral-network/chiral-network.git

# Show status
echo ""
echo "Git repository initialized successfully!"
echo ""
echo "To push to GitHub, run:"
echo "git push -u origin main"
echo ""
echo "Current git status:"
git status

echo ""
echo "Git remotes:"
git remote -v