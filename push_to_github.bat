@echo off
echo === XGen Protocol — GitHub Push ===
cd /d "G:\My Drive\Projects\XGenProtocol"
git init
git add .
git commit -m "Initial commit: XGen Protocol philosophy v0.3"
gh repo create ianus777/xgen-protocol --public --source=. --push --description "An open, federated, identity-verified communication protocol — structurally incapable of enshittification"
echo === Done! ===
pause
