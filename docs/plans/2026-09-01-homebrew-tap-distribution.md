## Goal

Ship `pomodoro-tui` (v0.2.0, Rust 2021, `iman1704/pomodoro-tui`) as a Homebrew package via the personal tap `iman1704/homebrew-tap` so users can `brew tap iman1704/tap && brew install pomodoro-tui` (or `brew install iman1704/tap/pomodoro-tui`) without an Apple Developer ID, notarization, or manual `cargo install`.

## Success Criteria

- `brew install iman1704/tap/pomodoro-tui` compiles from source and installs a working `pomodoro-tui` binary on macOS (Intel + Apple Silicon) and Linux (Linuxbrew).
- No quarantine/Gatekeeper prompt on the Homebrew-installed binary (source-build path avoids `com.apple.quarantine`; no Developer ID required).
- Tagging a new version (e.g. `v0.2.1`) in `pomodoro-tui` automatically or semi-automatically updates `homebrew-tap/Formula/pomodoro-tui.rb` with correct `url` + `sha256`.
- `brew audit --strict` and `brew test` pass for the formula.
- `README.md` and `TODO.md` installation docs reflect the new method.

## Context And Current Facts

- **Project:** `pomodoro-tui` v0.2.0 (`Cargo.toml:1-8`, MIT, `homepage`/`repository` = `https://github.com/iman1704/pomodoro-tui`). Pure Rust, deps `clap 4.5.37`, `crossterm 0.29.0`, `ratatui 0.29.0`, `tui-big-text 0.7.1`. No OpenSSL/native libs, so no extra `depends_on` beyond `rust`.
- **Current install:** `cargo build` / `cargo run` only; `README.md:15` has TODO for install instructions; `TODO.md:2` tracks Homebrew tap + CI.
- **Git state:** tags `v0.1.0`, `v0.2.0` exist, no `.github/workflows/` yet, no `Formula/` dir in main repo. `origin` = `iman1704/pomodoro-tui`, `upstream` = `xamcost/pomodoro`.
- **Tap repo:** `https://github.com/iman1704/homebrew-tap.git` exists and is reachable (`git ls-remote` returns `e3373aa` on `main`). User constraint: no Apple Developer account, so notarized Developer-ID signing and `bottle` notarization are out.
- **Homebrew research:** Verified `cargo install` + `depends_on "rust" => :build` + `*std_cargo_args` is the canonical Rust formula pattern (Formula Cookbook; multiple tap examples). `brew install --build-from-source` on a source formula does not receive a quarantine xattr, so Gatekeeper does not block it even when unsigned — binary-bottle taps also avoid quarantine but add CI complexity and cross-arch builds.
- **Codesign option:** `codesign --force --deep --sign -` (ad-hoc, identity `-`) creates a locally-sealed signature that prevents the "damaged" Gatekeeper error and reduces direct-download flow to a one-time "unidentified developer / Right-click → Open" prompt. It does NOT replace notarization; downloaded tarballs remain quarantine-flagged. For Homebrew source-build installs the xattr is never set, so ad-hoc signing is not needed for functionality.

## Constraints And Non-goals

- **Constraints:** No Apple Developer ID / notarization; must work on both macOS and Linux; must use `iman1704/homebrew-tap`; must not require users to disable Gatekeeper or run `xattr -cr` for the brew path; minimal ongoing maintenance.
- **Non-goals:** Publishing to `homebrew/core` (requires stricter notability + `brew test-bot` bottles and would reject a trivial fork); Windows support; auto-updating via `cargo-dist` / `GoReleaser` casks; notarized DMG/.app distribution.

## Key Decisions

| Decision | Recommendation | Why | Alternative rejected |
|---|---|---|---|
| **Distribution type** | Source-build formula (`cargo install`) as the primary and only initial artifact | Zero signing/notarization needed; brew strips quarantine for locally-compiled formulae; aligns with Homebrew audit expectations for Rust; trivial CI; already proven by dozens of taps (see Sources). No `bottle do` block needed. | **Pre-built binary bottle / `url` pointing to `releases/download/...tar.gz`** — faster install but requires per-arch CI matrix (x86_64/aarch64 × macOS/Linux), per-artifact `sha256`, and optionally ad-hoc `codesign -s -`. Without notarization, direct downloads still trigger Gatekeeper; bottles from a third-party tap also bypass quarantine, so signing buys little. Defer until install volume justifies 1–2 min compile cost saving. |
| **Codesign handling** | **Do not sign for the source-build path.** Document ad-hoc `codesign --sign -` only as Phase 2 if/when binary bottles are added (one `codesign --force --deep --sign - ./target/release/pomodoro-tui` step in the release job before `tar.gz`). | Brew-built binaries have no quarantine xattr, so Gatekeeper never evaluates them; adding a throwaway signature adds no security (ad-hoc = no identity) and complicates CI. | Always ad-hoc sign even for source build — wasted step, no user-visible benefit for brew path. Fully unsigned direct binary release without docs — leaves users hitting "damaged" error on direct GitHub downloads. |
| **Tap layout** | `homebrew-tap` repo with `Formula/pomodoro-tui.rb` (capitalized class `PomodoroTui`), `README.md`, `LICENSE`. Keep a mirror copy `pomodoro-tui/HomebrewFormula/pomodoro-tui.rb` or `Formula/` only as reference; `homebrew-tap` is source of truth. | Matches standard tap convention (see tap examples). `brew tap iman1704/tap` discovers all `*.rb` under `Formula/`. | Root-level `pomodoro-tui.rb` in tap — Homebrew still finds it but `Formula/` is conventional and expected by `brew test-bot`. Cask — wrong for CLI, and casks still hit Gatekeeper. |
| **Versioning & URL** | Formula `url "https://github.com/iman1704/pomodoro-tui/archive/refs/tags/vX.Y.Z.tar.gz"` with `sha256` of that archive, `license "MIT"`, `livecheck` block using `strategy :github_latest`. Tag `v0.2.0` drives v0.2.0 formula. | Standard GitHub archive URL is what `brew create` generates; no release asset to maintain. `livecheck` enables `brew livecheck` to detect new tags. | Point `url` at a hand-built `pomodoro-tui-v0.2.0-macos-arm64.tar.gz` — requires release workflow to produce + sign + upload that archive first. |
| **Update automation** | Two-repo automation: (A) `pomodoro-tui/.github/workflows/release.yml` that on tag push creates a GitHub Release; (B) `homebrew-tap/.github/workflows/update-formula.yml` that on `repository_dispatch` or cron `livecheck` bumps `url`/`sha256` (or a manual `brew bump-formula-pr` fallback). Simplest viable: start with (A) + manual `update-homebrew` dispatch script, add (B) after. | Decouples release from tap; works without PAT sharing between repos if using `workflow_dispatch` + `gh` CLI locally. | Single-repo `brew bump-formula-pr` that pushes directly to tap — needs cross-repo PAT and more permission scaffolding for v0.2.x. Fully manual `sed` edit on each release — error-prone and forgettable. |
| **Test block** | `test do; assert_match "Pomodoro", shell_output("#{bin}/pomodoro-tui --help"); assert_match "25", shell_output("#{bin}/pomodoro-tui --help"); end` plus `system "#{bin}/pomodoro-tui", "--version"` | `brew test` / `brew audit` require a non-trivial test; `--help`/`--version` are safe (no TUI launch). Avoids launching the interactive TUI in CI. | No test block — fails `brew audit --strict`. Running the binary interactively — hangs in CI. |

## Recommended Approach

**Phase 1 — Source-build formula (this plan).** One `PomodoroTui` formula that builds with `cargo install`. No code signing, no bottles. Users get a locally-compiled, unquarantined binary. CI only creates GitHub Releases from tags; the tap update is a lightweight workflow or a one-liner `brew bump-formula-pr` run locally.

**Phase 2 — Optional binary bottles (deferred).** If `cargo install` compile time (~60–120 s) becomes a complaint, add a matrix job (`macos-14` arm64, `macos-13` x86_64, `ubuntu-latest`) that runs `cargo build --release`, optionally `codesign --force --deep --sign - target/release/pomodoro-tui`, archives to `pomodoro-tui-${version}-${arch}.tar.gz`, uploads to the GitHub Release, and populates `bottle do` blocks in the formula via `brew bottle --json`. This keeps the source-build fallback and adds fast-path bottles. Because bottles are fetched by brew (no quarantine xattr), they also do not require notarization; ad-hoc signing is optional hygiene to avoid "damaged" on manual extraction.

Do not pursue a Cask, and do not pursue Developer-ID signing/notarization until/unless the user obtains an Apple Developer membership.

## Work Plan

### 1) Formula authoring — tap repo `iman1704/homebrew-tap`

- **Owner:** tap repo (`homebrew-tap`).
- **Files:**
  - `Formula/pomodoro-tui.rb` (new) — template:
    ```ruby
    class PomodoroTui < Formula
      desc "Simple Pomodoro timer with terminal UI (ratatui)"
      homepage "https://github.com/iman1704/pomodoro-tui"
      url "https://github.com/iman1704/pomodoro-tui/archive/refs/tags/v0.2.0.tar.gz"
      sha256 "<computed sha256 of v0.2.0 archive>"
      license "MIT"

      depends_on "rust" => :build

      def install
        system "cargo", "install", *std_cargo_args
      end

      livecheck do
        url :stable
        strategy :github_latest
      end

      test do
        assert_match "pomodoro", shell_output("#{bin}/pomodoro-tui --help")
        assert_match version.to_s, shell_output("#{bin}/pomodoro-tui --version")
      end
    end
    ```
  - `README.md` (update) — document `brew tap iman1704/tap` / `brew install pomodoro-tui` and `brew install iman1704/tap/pomodoro-tui`.
  - `LICENSE` — ensure MIT is referenced if not already present.
- **Compute SHA:** `curl -L https://github.com/iman1704/pomodoro-tui/archive/refs/tags/v0.2.0.tar.gz | shasum -a 256`.
- **Naming check:** formula file `pomodoro-tui.rb` → class `PomodoroTui` (Homebrew infers name from file; verify with `brew audit`).
- **Dep:** none beyond `rust` (no `openssl`, `pkg-config`, etc. — confirmed via `Cargo.lock` / `Cargo.toml`).

### 2) Release automation — main repo `iman1704/pomodoro-tui`

- **Owner:** `pomodoro-tui`.
- **Files:**
  - `.github/workflows/release.yml` (new) — trigger `on: push: tags: ["v*"]`, steps: `actions/checkout`, `softprops/action-gh-release` or `gh release create` to publish the tag's archive (GitHub auto-generates the tarball; no build artifact needed for source formula). Optionally add `cargo test` gate before release.
  - `.github/workflows/ci.yml` (new, optional but recommended) — `on: [push, pull_request]`, `cargo test` + `cargo build` to catch breakage before tag.
- **Version alignment:** `Cargo.toml` version `0.2.0` ↔ git tag `v0.2.0` (already matching). Document that bumping version requires updating both and pushing a tag.
- **Permissions:** `contents: write` for `GITHUB_TOKEN` to create releases.

### 3) Tap update automation — tap repo

- **Option A (minimal, recommended to start):** No workflow; document manual update:
  ```bash
  brew bump-formula-pr --tag=v0.2.1 --version=0.2.1 iman1704/tap/pomodoro-tui
  # or locally:
  curl -L https://github.com/iman1704/pomodoro-tui/archive/refs/tags/v0.2.1.tar.gz | shasum -a 256
  # then edit Formula/pomodoro-tui.rb
  ```
- **Option B (automated):** `.github/workflows/update-formula.yml` in `homebrew-tap`:
  - Trigger: `repository_dispatch` (sent from `pomodoro-tui` release workflow via PAT) **or** `schedule` + `workflow_dispatch`.
  - Action: checkout tap, compute new `sha256`, `sed` or `brew bump-formula-pr` equivalent, commit, push, optionally open PR.
  - Needs a PAT (`HOMEBREW_TAP_PAT`) stored as secret in `pomodoro-tui` repo — evaluate if worth the secret management for v0.2.x.
- **Decision:** Ship Option A first; add Option B after one manual release proves the flow.

### 4) Documentation — main repo

- **Files:**
  - `README.md:15` — replace TODO install section with:
    ```
    ## Installation
    ### Homebrew (macOS / Linux)
    brew tap iman1704/tap
    brew install pomodoro-tui
    # or in one line:
    brew install iman1704/tap/pomodoro-tui
    ### Cargo
    cargo install --locked pomodoro-tui  # once published, or
    cargo install --path . --locked
    ```
  - `TODO.md:2` — check off Homebrew items or link to this plan.
  - `AGENTS.md` — optionally note formula location and release steps (keep in sync with README per project rule).

### 5) Phase 2 — binary bottles (out of scope for initial ship, design retained)

- Add `pomodoro-tui/.github/workflows/bottle.yml` matrix build, `codesign --force --deep --sign - target/release/pomodoro-tui` (ad-hoc), `tar czf`, upload to release, then in tap run `brew bottle --json` to generate `bottle do` stanza. Not required for correctness; include as "Deferred" in the plan so the initial formula is not bottle-aware.

## Validation Plan

| Work unit | Command / check | Expected evidence |
|---|---|---|
| Formula syntax | `brew audit --strict iman1704/tap/pomodoro-tui` (after `brew tap iman1704/tap`) | No errors; warnings only for optional style nits |
| Style | `brew style Formula/pomodoro-tui.rb` | Pass |
| Build from source (macOS) | `brew install --build-from-source iman1704/tap/pomodoro-tui` | Compiles via `cargo install`, installs to `$(brew --prefix)/bin/pomodoro-tui` |
| Build from source (Linuxbrew, if available) | Same as above on Linux runner | Pass |
| Functional | `pomodoro-tui --help` and `pomodoro-tui --version` and `brew test iman1704/tap/pomodoro-tui` | Help shows `-w/--work`, `-b/--break`, `-n/--name`, `-i/--hide-image`; version matches formula `version` |
| Livecheck | `brew livecheck iman1704/tap/pomodoro-tui` | Detects `v0.2.0` (or latest tag) |
| No quarantine | `xattr -l $(which pomodoro-tui)` after brew install | No `com.apple.quarantine` attribute |
| CI (main repo) | `cargo test` locally and in `ci.yml` | All existing `lib.rs` unit tests pass; `cargo build --release` succeeds |
| Docs | Manual review of `README.md` install section | Homebrew instructions match actual tap name |
| Tap structure | `brew tap --list` shows `iman1704/tap`; `ls $(brew --repo iman1704/tap)/Formula/` | `pomodoro-tui.rb` present |

Highest-risk validation: `brew install --build-from-source` on a clean macOS runner (Apple Silicon) — this exercises `std_cargo_args`, the `Cargo.lock` (`--locked`), and the `rust` build dep together. Run it in CI or locally on an untapped machine before announcing.

## Risks / Rollback

- **Rust toolchain skew:** If tap users have an older `rust` Homebrew package, `--locked` may fail if `Cargo.lock` requires newer `rustc`. Mitigation: keep `Cargo.lock` committed, test with `brew install rust` latest; if needed pin `rust` version in formula or document `brew upgrade rust`.
- **SHA mismatch after tag move:** If a tag is force-pushed, the archive `sha256` changes and installs fail. Mitigation: never move tags; create new patch tags.
- **Tap name confusion:** Users may try `brew install pomodoro-tui` without tapping. Mitigation: document both `brew tap iman1704/tap && brew install pomodoro-tui` and `brew install iman1704/tap/pomodoro-tui`.
- **Quarantine misunderstanding:** Users who `curl -L .../releases/download/... | tar xz` outside brew will get quarantine. Mitigation: README should steer Homebrew users to the brew path; if Phase 2 binary tarballs are offered, note the one-time `xattr -dr com.apple.quarantine` / Right-click → Open workaround and that brew installs do not need it.
- **Cross-repo PAT for auto-bump:** Storing a PAT to push from `pomodoro-tui` to `homebrew-tap` increases secret handling. Mitigation: start with manual bump; defer automation.
- **Rollback:** Delete or revert `Formula/pomodoro-tui.rb` in `homebrew-tap`; users can `brew uninstall pomodoro-tui` and fall back to `cargo install --path .`. No data migration involved.

## Open Questions

- **Q1:** Should the initial release automate tap updates (Option B) or stay manual (Option A)? **Assumption:** Option A (manual `brew bump-formula-pr` / SHA edit) for the first release; automate after one successful manual cycle. Confirm preference.
- **Q2:** Do you want a `cargo publish` to crates.io in parallel, or is Homebrew the sole distribution channel for now? (Affects whether `cargo install pomodoro-tui` is also documented as a crates.io install.)
- **Q3:** Should the formula file also be checked into `pomodoro-tui` (e.g. `HomebrewFormula/pomodoro-tui.rb`) as a reference, or should `homebrew-tap` be the single source of truth? **Assumption:** Single source of truth in `homebrew-tap`; main repo only holds CI workflows.

## Sources

- https://docs.brew.sh/Formula-Cookbook — canonical Rust formula pattern (`depends_on "rust" => :build`, `system "cargo", "install", *std_cargo_args`, `--locked` via `std_cargo_args`, `livecheck` block)
- https://docs.brew.sh/How-To-Open-a-Homebrew-Pull-Request — tap vs `homebrew/core` distinction and `brew bump-formula-pr` workflow
- Tap structure & Rust `cargo install` examples inspected via web search (choose/exarch, cotoba, peeroxide, codedeviate/homebrew-cli) confirming `Formula/*.rb` layout and `depends_on "rust" => :build`
- Ad-hoc codesign Gatekeeper behavior inspected via multiple GitHub commit messages/issues (Tauri `signingIdentity "-"`, `codesign --force --deep --sign -`) and tap docs noting Homebrew bottles/source builds are not quarantined and do not require Developer ID
- https://github.com/iman1704/homebrew-tap — verified reachable tap repo (`main` at `e3373aa`)
