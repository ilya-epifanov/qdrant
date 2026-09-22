# Patched Edge SDK provenance

This branch packages the published `qdrant-edge` 0.8.0 crate with one upstream fix, the reviewed public feature-initialization API patch and a consuming terminal-finish API. It is maintained for the consuming `grayce/entity-linking` project, not an upstream-supported Edge release.

```text
crates.io qdrant-edge 0.8.0
    -> unchanged standalone package import
    -> preserved edge-layout.patch
    -> comment-only modification-notice.patch
    -> public-feature-flags.patch
    -> terminal-finish.patch
    -> immutable consuming-project Gitlink at third-party/qdrant
```

## Source identities

- Archive: https://static.crates.io/crates/qdrant-edge/qdrant-edge-0.8.0.crate
- Archive SHA256: `0b8072302c87506a34bffec9bc16dbdcd36df8ab1321406b6e141530348c7e54`, authenticated against the registry in the qualification experiment.
- Exact import commit: `6e0e40eb79ffc83c0df9085493b099a9bdc8a428`, containing all 1,074 archive files without changes.
- Upstream fix: https://github.com/qdrant/qdrant/commit/bbd54cf6f01fddbf63c779d682391ab8a6e885cf, PR 10239, authored by generall, Andrey Vasnetsov.
- Original upstream patch: `bbd54cf.patch`, SHA256 `ca35464ee0f5b9faa1cec656221587d26f3f576bb02831a030b0e4186fa48b53`.
- Applied patch: `edge-layout.patch`, SHA256 `071f59753e7ce3cecb22820f5d641c3572683052b9fbe75637562b4a6be9f415`.
- Qualified `src/segment/index/hnsw_index/links_container.rs` before the packaging notice, SHA256: `cee39f03c210bcac53c12737b2ed0d7cdd4558e7acc1bb554e69fd247257f900`.
- Comment-only packaging patch: `modification-notice.patch`, SHA256 `239667857081a89a7d7150c706dad4c4375c0dca76be5364f33cab697d5c7052`.
- Distributed `src/segment/index/hnsw_index/links_container.rs` after the notice, SHA256: `92471e866dfb705316822607fe7aae3776b988fd760ca47a7042dd3dcf9cb021`.

The dense engine-source change is `LinksContainer`, including upstream's duplicate-vector regression test. The applied patch changes the upstream monorepo paths and one unchanged import context to match the published flattened package. It does not change the upstream algorithm.

The fork is `git@github.com:ilya-epifanov/qdrant.git`. Its dedicated orphan branch is `edge-sdk-0.8.0-bbd54cf6`. The existing `master` at `6ab21cac18ebb6f4ae29102c7f8f5cc11affd5de` remains untouched. The branch starts from the authenticated published package rather than the monorepo because qualification used that standalone package and dependency metadata. Both Cargo manifests, the lockfile, build scripts and bundled sources remain as published. The manifest's version is still 0.8.0; the immutable Gitlink and this patch identity distinguish it from unpatched 0.8.0. Do not use the moving branch as a release identity.

## License and attribution

The package declares Apache-2.0 and credits the Qdrant Team in its unchanged manifest. The archive did not include a root license file. `../LICENSE` preserves the license fetched from https://raw.githubusercontent.com/qdrant/qdrant/bbd54cf6f01fddbf63c779d682391ab8a6e885cf/LICENSE, SHA256 `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`. The two preserved upstream patches retain upstream authorship and exact bytes. The separate `modification-notice.patch` adds the file-level change notice required by Apache-2.0 section 4(b). It prepends exactly the following two comment lines and one blank line, with LF line endings, without changing qualified algorithm bytes:

```rust
// Modified from published qdrant-edge 0.8.0 by porting upstream fix
// bbd54cf6f01fddbf63c779d682391ab8a6e885cf. See provenance/README.md.

```

Existing bundled source notices remain unchanged. This document does not claim a complete transitive-license audit.

## Reproduce and verify

Run from the SDK checkout with `PROJECT_ROOT` set to the consuming checkout. Use a new directory name rather than overwriting an earlier verification. Required tools are curl, shasum, tar, patch and diff.

```sh
set -eu
sdk="$PWD"
work="$PROJECT_ROOT/.tmp/edge-sdk-source-verification"
mkdir "$work"
curl --fail --location https://static.crates.io/crates/qdrant-edge/qdrant-edge-0.8.0.crate -o "$work/source.crate"
printf '%s  %s\n' 0b8072302c87506a34bffec9bc16dbdcd36df8ab1321406b6e141530348c7e54 "$work/source.crate" | shasum -a 256 -c -
tar -xzf "$work/source.crate" -C "$work"
(cd "$work/qdrant-edge-0.8.0" && patch -p1 --fuzz=0 < "$sdk/provenance/edge-layout.patch")
printf '%s  %s\n' cee39f03c210bcac53c12737b2ed0d7cdd4558e7acc1bb554e69fd247257f900 "$work/qdrant-edge-0.8.0/src/segment/index/hnsw_index/links_container.rs" | shasum -a 256 -c -
(cd "$work/qdrant-edge-0.8.0" && patch -p1 --fuzz=0 < "$sdk/provenance/modification-notice.patch")
printf '%s  %s\n' 92471e866dfb705316822607fe7aae3776b988fd760ca47a7042dd3dcf9cb021 "$work/qdrant-edge-0.8.0/src/segment/index/hnsw_index/links_container.rs" | shasum -a 256 -c -
printf '%s  %s\n' 72a36d1788bd4c0da4ef47af049d872d37ed3400e439f89cde45b2be2fd9d96d "$sdk/provenance/public-feature-flags.patch" | shasum -a 256 -c -
printf '%s  %s\n' e55d3ec968fcfbc6092c121e82d33ea982f82ed198b5c47254874ac125fcb755 "$work/qdrant-edge-0.8.0/src/edge/reexports.rs" | shasum -a 256 -c -
(cd "$work/qdrant-edge-0.8.0" && patch -p1 --fuzz=0 < "$sdk/provenance/public-feature-flags.patch")
printf '%s  %s\n' 8e24184b32af1a699feb4e3bf344ebefce0118093e2560d7380ecf5a721d4f6f "$work/qdrant-edge-0.8.0/src/edge/reexports.rs" | shasum -a 256 -c -
printf '%s  %s\n' ad683511d631a5e50af9395c6a4807032a1ae26f101361df50c3401395397564 "$sdk/provenance/terminal-finish.patch" | shasum -a 256 -c -
(cd "$work/qdrant-edge-0.8.0" && patch -p1 --fuzz=0 < "$sdk/provenance/terminal-finish.patch")
diff -r -x .git -x LICENSE -x provenance "$work/qdrant-edge-0.8.0" "$sdk"
```

The dense and notice patch commands must report only `patching file 'src/segment/index/hnsw_index/links_container.rs'`, with no offsets, fuzz, rejects or other output. The sealed patch command must report only `patching file 'src/edge/reexports.rs'`, with no offsets, fuzz, rejects or other output. Reject extra files. The only allowed additions to the archive are root `LICENSE`, `provenance/README.md`, `provenance/bbd54cf.patch`, `provenance/edge-layout.patch`, `provenance/modification-notice.patch`, `provenance/public-feature-flags.patch` and `provenance/terminal-finish.patch`; `.git` is checkout metadata. Confirm this exact addition set separately from the recursive diff, which skips the provenance directory. There are no generated build files in the SDK checkout. These diff exceptions describe source reconstruction, not repository scanner policy.

Removing only the exact declared notice prefix from the distributed LinksContainer file reproduces the qualified file byte-for-byte. Apart from the seven declared modified archive files, all 1,067 imported files remain unchanged from the published archive. That Cargo-cache-derived copy additionally has `.cargo-ok`, a Cargo extraction marker absent from the published archive and this Git package.

## Build contract and maintenance

The consuming project's maintainers own this packaging branch, patch review, qualification and Gitlink updates. Keep the patches limited to the dense fix, public feature-initialization exports and consuming terminal finish. For each update, authenticate the replacement archive, compare all package files, preserve license notices, rerun the consuming project's qualification commands, and review an immutable commit before changing the Gitlink. A newer upstream release is not automatically compatible. Rebuild graphs created by the defective algorithm from retained values; a patched reader does not repair existing graph links.

Existing qualification used Rust 1.98.1 and locked consumer dependencies on macOS and native Linux arm64. Exact source identity is not a build run in this checkout. The SDK's own unit-test harness has not been run here. Native amd64 qualification remains a consuming-project gate. Production storage integration and deployment are not authorized by this package.

The SDK has no Justfile. In this nested development location, `cargo metadata --no-deps --offline --format-version 1` fails because Cargo discovers the enclosing entity-linking workspace and this package is not a member. Do not rewrite the published manifest to conceal that failure. The consuming project must resolve workspace placement explicitly before relying on direct SDK commands. A standalone checkout outside an enclosing Cargo workspace avoids that discovery conflict, but still requires build qualification. A consumer controls its own dependency lock; this package's Cargo.lock does not lock downstream builds.

## Public feature initialization

`public-feature-flags.patch` is preserved byte-for-byte from consuming-project experiment `e6ddbe29ea8261088a9bc2a69c761ccb5c716843`, path `experiments/edge-sealed/patches/public-feature-flags.patch`. Its SHA256 is `72a36d1788bd4c0da4ef47af049d872d37ed3400e439f89cde45b2be2fd9d96d`.

The original `src/edge/reexports.rs` SHA256 is `e55d3ec968fcfbc6092c121e82d33ea982f82ed198b5c47254874ac125fcb755`; the distributed result is `8e24184b32af1a699feb4e3bf344ebefce0118093e2560d7380ecf5a721d4f6f`. Before terminal finish, this was the second modified archive file, with six explicit provenance/license additions and 1,080 distributed files. The patch retains its reviewed modification notice unchanged. It only exports existing `FeatureFlags` and `init_feature_flags`; defaults and normalization are unchanged.

Initialize feature flags once, before opening any shards. Initialization is process-global, not a per-reader option. The reviewed sealed experiment uses this API to set only `write_segment_manifest=true`, preserving all other defaults, including `incremental_hnsw_building=true`. It does not fix raw partial follower opens. Production admission, publication, recovery, graph/config correspondence and request pins remain separate gates. Tickets08/09 remain gated. The prior experiment review accepted experiment qualification, not production adoption or native AMD64 qualification of this distribution.

## Consuming terminal finish

`terminal-finish.patch` adds `EdgeShard::finish` and independent retired-WAL, current-WAL, creator and segment outcomes. Its SHA256 is `ad683511d631a5e50af9395c6a4807032a1ae26f101361df50c3401395397564`. Its input is packaging commit `c00671f403d62e3657eefc69cf2b94e0864feea0`. The five changed files are `src/edge/edge_shard/mod.rs`, `src/edge/mod.rs`, `src/shard/wal.rs`, `src/wal/mod.rs` and `src/wal/segment_creator.rs`. Each carries an Apache-2.0 modification notice. The reconstruction command must patch exactly those files without offsets, fuzz or rejects. There are now seven modified archive files, seven explicit provenance/license additions and 1,081 distributed files. Published manifests and lockfile remain unchanged.

A report returns only after the shard is destroyed. Returned phase errors do not skip later phases. The report preserves original IO errors, worker panic payloads and SDK segment errors. It does not restore underlying segment IO causes that existing SDK conversions already discarded. Synchronous panics propagate without a report. Finish disables the redundant EdgeShard Drop flush before starting work; no new Drop join or executor is added. Escaped views remain the caller's responsibility.

The consuming project's `tools/qdrant-qualification/Justfile` owns terminal-finish qualification. It builds an uninstrumented copy and a separate fault-instrumented copy with fresh fixtures and locked SDK dependencies. Instrumentation and qualification examples are not distributed SDK code. The SDK's upstream cfg(test) baseline fails compilation with missing test modules and other errors; these tests have not been repaired or declared passing. Application ownership, seal/publication behavior, full CI and independent review remain separate acceptance gates. This local source checkpoint does not authorize remote publication.
