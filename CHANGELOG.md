+++
title = "Changelog"
description = "Changelog"
weight = 3
+++

## 0.14.0 (2024-01-13)

- Docker compose  by @pxp9 in https://github.com/ayrat555/el_monitorro/pull/352
- update rust and deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/353
- fix: handle invalid publication dates by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/354
- update frankenstein to 0.29.2 by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/355
- Frankenstein 0.30.0 by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/356

## 0.12.1 (2023-09-29)

- chore(cargo): bump thiserror from 1.0.35 to 1.0.36 by @dependabot in https://github.com/ayrat555/el_monitorro/pull/268
- Update fang from 0.9 to 0.10 by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/269
- add BotCommand enum by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/270
- update diesel and handlebars by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/272
- chore(cargo): bump serde_json from 1.0.85 to 1.0.86 by @dependabot in https://github.com/ayrat555/el_monitorro/pull/273
- Specify the postgres version when running tests by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/274
- update rust version in the dockerfile to 1.65.0 by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/279
- handle invalid links by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/280
- Fix retries query for postgres 15 by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/282
- chore(cargo): bump serde from 1.0.147 to 1.0.148 by @dependabot in https://github.com/ayrat555/el_monitorro/pull/283
- add inline keyboard for help command by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/288
- add command to close/remove keyboards and messages by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/289
- support `author` in templates by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/294
- support authors from dublincore by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/295
- add `preview_enabled` to chats by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/296
- feed keyboards by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/298
- improve logs by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/299
- Process commands in conversational style by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/301
- CommandsKeyboard improvements by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/302
- Display only read-only commands for feed keyboards in public chats by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/303
- update deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/309
- support topics by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/310
- update fang by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/311
- Compare content fields when syncronizing by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/312
- ci(actions): bump actions/checkout from 2 to 3 by @dependabot in https://github.com/ayrat555/el_monitorro/pull/313
- update rust version and deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/322
- update rust (1.68.2) and deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/326
- update frankenstein (0.25.0) by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/329
- update aho-corasick by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/331
- update crates by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/333
- bump deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/338
- update rust version and deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/339
- update frankenstein by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/341
- update deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/342
- update rust version by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/343
- update deps by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/345
- update fang and frankenstein by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/347
- update rust to 1.72 in dockerfile by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/348
- use OnceLock from std instead of the `once_cell` crate by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/349
- update crates by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/350
- update frankenstein by @ayrat555 in https://github.com/ayrat555/el_monitorro/pull/351

## 0.12.0 (2023-02-05)

- Add inline keyboard for help command ([#288](https://github.com/ayrat555/el_monitorro/pull/288))
- Add command to close/remove keyboards and messages ([#289](https://github.com/ayrat555/el_monitorro/pull/289))
- Support `author` in templates ([#294](https://github.com/ayrat555/el_monitorro/pull/294))
- Support authors from dublincore ([#295](https://github.com/ayrat555/el_monitorro/pull/295))
- Add `preview_enabled` to chats ([#296](https://github.com/ayrat555/el_monitorro/pull/296))
- Feed keyboards ([#298](https://github.com/ayrat555/el_monitorro/pull/298))
- Process commands in conversational style ([#301](https://github.com/ayrat555/el_monitorro/pull/301))
- CommandsKeyboard improvements ([#302](https://github.com/ayrat555/el_monitorro/pull/302))
- Display only read-only commands for feed keyboards in public chats ([#303](https://github.com/ayrat555/el_monitorro/pull/303))
- Update rust version and dependencies

## 0.11.0 (2022-09-24)

### Improved

- Optimize filter word matching using the ahocorasick crate ([#263](https://github.com/ayrat555/el_monitorro/pull/263))
- Configure the max number of filter words with the env var ([#264](https://github.com/ayrat555/el_monitorro/pull/264))
- Re-use the http client for all requests ([#265](https://github.com/ayrat555/el_monitorro/pull/265))
- Parse links along with their content   ([#266](https://github.com/ayrat555/el_monitorro/pull/266))

### Fixed

- Fix long links in templates ([#257](https://github.com/ayrat555/el_monitorro/pull/257))

## 0.10.0 (2022-09-04)

### Added
- Add template helpers (html links, text formatting) ([#244](https://github.com/ayrat555/el_monitorro/pull/244))

### Improved
- Update diesel crate to 2.0 version. ([#247](https://github.com/ayrat555/el_monitorro/pull/247))
- Update fang crate to 0.9.0 version. ([#247](https://github.com/ayrat555/el_monitorro/pull/247))
- Docker push automatization. ([#239](https://github.com/ayrat555/el_monitorro/pull/239))

## 0.9.0 (2022-08-01)

- Do not reply to message replies ([#219](https://github.com/ayrat555/el_monitorro/pull/219))
- Allow spaces in local filters ([#222](https://github.com/ayrat555/el_monitorro/pull/222))
- Use rayon instead of tokio for processing user commands ([#231](https://github.com/ayrat555/el_monitorro/pull/231))

## 0.8.0 (2022-06-01)

- Add global commands for filters ([#214](https://github.com/ayrat555/el_monitorro/pull/214))

## 0.7.0 (2022-05-14)

- Update frankenstein to 0.15 by @ayrat555 in [#212](https://github.com/ayrat555/el_monitorro/pull/212)
- Sync and deliver during subscription by @ayrat555 in [#213](https://github.com/ayrat555/el_monitorro/pull/213)

## 0.6.0 (2022-04-02)

- Add ability to change primary key of feed items table ([#206](https://github.com/ayrat555/el_monitorro/pull/206))

## 0.5.0 (2022-02-03)

- Remove custom templates (handlebars can be used directly) ([#196](https://github.com/ayrat555/el_monitorro/pull/196))
- Add `remove_filter`, `remove_template`, `remove_global_template` ([#197](https://github.com/ayrat555/el_monitorro/pull/197))

## 0.4.0 (2022-01-22)

- Migrate from html2text to nanohtml2text ([#189](https://github.com/ayrat555/el_monitorro/pull/189))

## 0.3.0 (2022-01-10)

- Change the primary key of feed_items from `(feed_id, title, link)` to `(feed_id, content_hash)` - [#183](https://github.com/ayrat555/el_monitorro/pull/183), [#184](https://github.com/ayrat555/el_monitorro/pull/184), [#185](https://github.com/ayrat555/el_monitorro/pull/185), [#186](https://github.com/ayrat555/el_monitorro/pull/186)

## 0.2.6 (2022-01-03)

- Update frankenstein

## 0.2.5 (2021-12-31)

- if `OWNER_TELEGRAM_ID` is set, process commands only from this user - [#8bd0253d8f7ae7fb0ac70c](https://github.com/ayrat555/el_monitorro/commit/8bd0253d8f7ae7fb0ac70cc7bafa7df3d8096f33)
- Update deps

## 0.2.4 (2021-12-05)

- Update frankenstein

## 0.2.3 (2021-11-09)

- Bump deps (tokio, serde_json, feed-rs, diesel, isahc, frankenstein)
- Set retention mode for workers - [RemoveAll](https://github.com/ayrat555/el_monitorro/commit/b363e7d3fce90534b4ebfacf72f9349060bdfba0)

## 0.2.2 (2021-09-12)

- Add exponential backoff for failing feeds ([#155](https://github.com/ayrat555/el_monitorro/pull/155))
- Add admin stats (/info) commands ([#156](https://github.com/ayrat555/el_monitorro/pull/156), [#157](https://github.com/ayrat555/el_monitorro/pull/157))
- Decrease docker image size ([#c93b02233bff](https://github.com/ayrat555/el_monitorro/commit/c93b02233bff8adeed77ffe32f2a5215006ac108))

## 0.2.1 (2021-09-08)

- always update synced_at in feeds ([#154](https://github.com/ayrat555/el_monitorro/pull/154))

## 0.2.0 (2021-09-07)

### Major performance improvements

- migrate all background processing to [fang](https://github.com/ayrat555/fang)
- allow to start all services from the main binary with `ALL_BINARIES` env var
- maintain db pool for processing user commands

## 0.1.3 (2021-06-22)

### Chores

- Migrate to the frankenstein telegram library ([#100](https://github.com/ayrat555/el_monitorro/pull/100))
- Update deps ([#101](https://github.com/ayrat555/el_monitorro/pull/101))
- Fix clippy warnings ([#102](https://github.com/ayrat555/el_monitorro/pull/102))

## 0.1.2 (2021-04-19)

### Enhancements

- Allow docker to run all commands at once ([#99](https://github.com/ayrat555/el_monitorro/pull/99))

## 0.1.1 (2021-03-10)

### Enhancements

- Add subscription filters ([#90](https://github.com/ayrat555/el_monitorro/pull/90))
- Add negated subscription filters ([#93](https://github.com/ayrat555/el_monitorro/pull/93))

### Bugfixes

- Fix new update types ([#95](https://github.com/ayrat555/el_monitorro/pull/95))
- Return error if the feed is not found during sync ([#89](https://github.com/ayrat555/el_monitorro/pull/89))

### Chores

- Add index for feed_id in feed_items ([2e0ef3105](https://github.com/ayrat555/el_monitorro/commit/2e0ef310528ff050eb8786d561171a709940f6c6))
- Add new delivery error type ([ce11ee487](https://github.com/ayrat555/el_monitorro/commit/ce11ee487f89f123efb98390f1159d2ea54e9e47))

## 0.1.0 (2021-02-14)

- The first release on docker hub :tada:
