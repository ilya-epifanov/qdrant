# Patched Edge SDK provenance

This branch packages the published `qdrant-edge` 0.8.0 crate with one upstream fix. It is maintained for the consuming `grayce/entity-linking` project, not an upstream-supported Edge release.

```text
crates.io qdrant-edge 0.8.0
    -> unchanged standalone package import
    -> preserved edge-layout.patch
    -> immutable consuming-project Gitlink at third-party/qdrant
```

## Source identities

- Archive: https://static.crates.io/crates/qdrant-edge/qdrant-edge-0.8.0.crate
- Archive SHA256: `0b8072302c87506a34bffec9bc16dbdcd36df8ab1321406b6e141530348c7e54`, authenticated against the registry in the qualification experiment.
- Exact import commit: `6e0e40eb79ffc83c0df9085493b099a9bdc8a428`, containing all 1,074 archive files without changes.
- Upstream fix: https://github.com/qdrant/qdrant/commit/bbd54cf6f01fddbf63c779d682391ab8a6e885cf, PR 10239, authored by generall, Andrey Vasnetsov.
- Original upstream patch: `bbd54cf.patch`, SHA256 `ca35464ee0f5b9faa1cec656221587d26f3f576bb02831a030b0e4186fa48b53`.
- Applied patch: `edge-layout.patch`, SHA256 `071f59753e7ce3cecb22820f5d641c3572683052b9fbe75637562b4a6be9f415`.
- Patched `src/segment/index/hnsw_index/links_container.rs` SHA256: `cee39f03c210bcac53c12737b2ed0d7cdd4558e7acc1bb554e69fd247257f900`.

The only engine-source change is `LinksContainer`, including upstream's duplicate-vector regression test. The applied patch changes the upstream monorepo paths and one unchanged import context to match the published flattened package. It does not change the upstream algorithm.

The fork is `git@github.com:ilya-epifanov/qdrant.git`. Its dedicated orphan branch is `edge-sdk-0.8.0-bbd54cf6`. The existing `master` at `6ab21cac18ebb6f4ae29102c7f8f5cc11affd5de` remains untouched. The branch starts from the authenticated published package rather than the monorepo because qualification used that standalone package and dependency metadata. Both Cargo manifests, the lockfile, build scripts and bundled sources remain as published. The manifest's version is still 0.8.0; the immutable Gitlink and this patch identity distinguish it from unpatched 0.8.0. Do not use the moving branch as a release identity.

## License and attribution

The package declares Apache-2.0 and credits the Qdrant Team in its unchanged manifest. The archive did not include a root license file. `../LICENSE` preserves the license fetched from https://raw.githubusercontent.com/qdrant/qdrant/bbd54cf6f01fddbf63c779d682391ab8a6e885cf/LICENSE, SHA256 `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`. The two preserved patches retain upstream authorship. Existing bundled source notices remain unchanged. This document records the local modification; it does not claim a complete transitive-license audit.

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
diff -r -x .git -x LICENSE -x provenance "$work/qdrant-edge-0.8.0" "$sdk"
```

Only `.git`, the added root `LICENSE` and this provenance directory are outside the reconstructed archive. There are no generated build files in the SDK checkout. These diff exceptions describe source reconstruction, not repository scanner policy.

Qualification source comparison also matched all 1,074 package files against the prior patched vendor copy. That Cargo-cache-derived copy additionally has `.cargo-ok`, a Cargo extraction marker absent from the published archive and this Git package.

## Build contract and maintenance

The consuming project's maintainers own this packaging branch, patch review, qualification and Gitlink updates. Keep the patch limited to this fix. For each update, authenticate the replacement archive, compare all package files, preserve license notices, rerun the consuming project's qualification commands, and review an immutable commit before changing the Gitlink. A newer upstream release is not automatically compatible. Rebuild graphs created by the defective algorithm from retained values; a patched reader does not repair existing graph links.

Existing qualification used Rust 1.98.1 and locked consumer dependencies on macOS and native Linux arm64. Exact source identity is not a build run in this checkout. The SDK's own unit-test harness has not been run here. Native amd64 qualification remains a consuming-project gate. Production storage integration and deployment are not authorized by this package.

The SDK has no Justfile. In this nested development location, `cargo metadata --no-deps --offline --format-version 1` fails because Cargo discovers the enclosing entity-linking workspace and this package is not a member. Do not rewrite the published manifest to conceal that failure. The consuming project must resolve workspace placement explicitly before relying on direct SDK commands. A standalone checkout outside an enclosing Cargo workspace avoids that discovery conflict, but still requires build qualification. A consumer controls its own dependency lock; this package's Cargo.lock does not lock downstream builds.
