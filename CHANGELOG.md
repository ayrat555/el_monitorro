# Changelog

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
