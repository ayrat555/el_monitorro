+++
title = "Changelog"
description = "Changelog"
weight = 3
+++

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
