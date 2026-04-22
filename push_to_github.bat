@echo off
echo === XGen Protocol — GitHub Push ===
cd /d "G:\My Drive\Projects\XGenProtocol"
git init
git add .
git commit -m "Initial commit: XGen Protocol philosophy v0.3"
git branch -M main
git remote add origin https://github.com/ianus777/XGenProtocol.git
git push -u origin main
echo === Done! ===
pause
