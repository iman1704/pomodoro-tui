# Releasing a new version

How to ship an update to Homebrew (source-build, no signing needed).

## 1. Bump and tag `pomodoro-tui`

```bash
# edit Cargo.toml -> version = "0.2.1"
cargo test
git add Cargo.toml Cargo.lock
git commit -m "chore: bump to 0.2.1"
git push origin master

git tag v0.2.1
git push origin v0.2.1
# GitHub Action creates the Release automatically
```

## 2. Update the tap

```bash
# clone once
git clone https://github.com/iman1704/homebrew-tap.git ~/homebrew-tap
cd ~/homebrew-tap

# get new SHA
curl -sL https://github.com/iman1704/pomodoro-tui/archive/refs/tags/v0.2.1.tar.gz | shasum -a 256

# edit Formula/pomodoro-tui.rb:
#   url "https://github.com/iman1704/pomodoro-tui/archive/refs/tags/v0.2.1.tar.gz"
#   sha256 "<paste new SHA>"

git add Formula/pomodoro-tui.rb
git commit -m "chore: bump pomodoro-tui to 0.2.1"
git push origin main
```

Or: `brew bump-formula-pr --tag=v0.2.1 --version=0.2.1 iman1704/tap/pomodoro-tui`

## 3. Verify

```bash
brew update && brew upgrade pomodoro-tui
pomodoro-tui --version
```
