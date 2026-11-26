# xrf-cache

An application's caches on disk: what rebuilds from nothing, such as the renderer's compiled pipelines, kept between runs
in one folder per client and tidied by age and size. The desktop app keeps its caches here, and any other native host
can build the same caches over a folder of its own.

## The caches

`AppCache::new(root)` stands over a folder of caches and nothing else: a tidy removes files within its client folders,
so `root` never holds an application's saved files, logs or exports. `get_store(name)` answers one client's
`AppCacheStore`, the folder `root/<name>`.

`tidy(policy, is_forced)` removes the entries unused longer than the policy's days, then the least recently used until
the rest fit its size limit, and answers an `AppCacheTidy` of what it removed and kept:

- At most once a day unless forced: the `.tidied` file's modification time is when the last tidy ran.
- Under a non-blocking lock (`.lock`): a tidy another instance holds is left to it, as Cargo's cache cleanup skips a
  locked cache. An entry that cannot be removed, open elsewhere, is kept.
- `AppCache::is_own_file(name)` says which files at the root are that bookkeeping rather than a client's.

`AppCachePolicy` sets the age and the limit; `DEFAULT_APP_CACHE_POLICY` is a week unused and five gibibytes, and reaches
the app's page as a generated constant.

## One client's cache

An `AppCacheStore` is a folder of entries, each a file written whole (`write`) or not at all. `read` answers an
entry's bytes, or none where it is not kept or cannot be read. An entry's modification time is when it was last used:
set as it is written, and moved on by a read once it stands an hour stale, so the tidy needs no index kept in step
across processes. Each read and write is logged with its size and how long ago the entry was last used.

## Checks

Run `cargo test --locked -p xrf-cache` from the repository root.
