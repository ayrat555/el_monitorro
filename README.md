+++
title = "El Monitorro"
sort_by = "weight"
+++

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/AyratBadykov)

# El Monitorro

El Monitorro is RSS, Atom and JSON feed reader as a Telegram bot.

It's available at [@el_monitorro_bot](https://t.me/el_monitorro_bot).

# Usage

### Commands

```
/start - show the bot's description and contact information

/subscribe url - subscribe to feed

/unsubscribe url - unsubscribe from feed

/list_subscriptions - list your subscriptions

/help - show available commands

/set_timezone - set your timezone. All received dates will be converted to this timezone. It should be offset in minutes from UTC. For example, if you live in UTC +10 timezone, offset is equal to 600

/get_timezone - get your timezone

/set_template url template - set a template for all received items for the specified subscription. All new updates will be converted to the format defined by this subscription. Supported fields you can use for templates:
- bot_feed_name - name of the feed
- bot_feed_link - url of the feed
- bot_item_name - name of the item
- bot_item_link - url of the item
- bot_item_description - description of the item
- bot_date - publication date of the feed

Example: /set_template https://www.badykov.com/feed.xml {{bot_feed_name}}


    {{bot_item_name}}


    {{bot_date}}


    {{bot_item_link}}

Also, there are some helpers for templates:
- `substring` helper that can be used to limit the number of characters. For example, {{substring bot_item_description 100}}
- `create_link` helper. This helper creates an html link. For example, {{create_link bot_item_name bot_item_link}} or {{create_link "custom_name" bot_item_link}}
- `italic` helper. Usage: {{italic bot_item_description}}
- `bold` helper. Usage:  {{bold bot_item_name}}

/get_template url - get a template for the subscription

/remove_template url - remove the template

/set_global_template - set global template. This template will be used for all subscriptions. If the subscription has its own template, the subscription template will be used. See /set_template for available fields.

/remove_global_template - remove global template

/get_global_template - get global template

/get_filter url - get a filter for the subscription

/set_filter url filter - set filter, for example, /set_filter https://www.badykov.com/feed.xml telegram,bots. You'll start receiving posts only containing words in the filter. Use `!word` to stop receiving messages containing the specified `word`. You can combine regular filter words with ! filter words. For example, `!bot,telegram`

/remove_filter url - remove filter

/set_global_filter filter - set global filter

/get_global_filter - get a global filter

/remove_global_filter - remove global filter

/info - shows the number of subscriptions and chats. it's available only for the admin (`ADMIN_TELEGRAM_ID`)

/set_content_fields url fields - changes content hash fields of the specified feed. it's available only for the admin (`ADMIN_TELEGRAM_ID`).
Example: /set_content_fields https://www.badykov.com/feed.xml author,title

By default content hash is calculated from title and url.

Available fields:
    - link
    - title
    - publication_date
    - guid
    - description
    - author
```

### Common info

- Feed updates check interval is 1 minute.
- Unread items delivery interval is 1 minute.
- The number of subscriptions is limited to 20.

The bot works in private chats, groups and channels. A couple of channels created with el monitorro:

- https://t.me/emacs_posts - Emacs News and Posts
- https://t.me/metacritic_tv - Latest Tv Show Metascores on Metacritic

# Setup

## Manual setup

You can deploy your instance of `el_monitorro` by:

1. Set postgres db url (`DATABASE_URL`) and telegram bot token (`TELEGRAM_BOT_TOKEN`) to `.env` file in the root directory. For example:

```
DATABASE_URL=postgres://admin:pass@localhost/el_monitorro
TELEGRAM_BOT_TOKEN=MYTOKEN
```

2. Setup database by running:

```
diesel database setup
```

You'll need diesel-cli for this

3. Start the bot

- Start the command bot

```
RUST_LOG=info RUST_BACKTRACE=1 cargo run --bin el_monitorro
```
- Start the sync binary

```
RUST_LOG=info RUST_BACKTRACE=1 cargo run --bin sync
```

- Start the delivery binary

```
RUST_LOG=info RUST_BACKTRACE=1 cargo run --bin deliver
```

- If you don't want to store all feed items that were synced and feeds without any subscriptions, start the cleaner binary

```
RUST_LOG=info RUST_BACKTRACE=1 cargo run --bin cleaner
```

### Running all services from a single binary

Set `ALL_BINARIES` to run all binaries (clean, commands, deliver, sync) in the same binary:

```
ALL_BINARIES=true
```

### Configuration

All configuration is done through env variables

| Name                     | Required | Default value | Example / Description                                                                                                                                                               |
|--------------------------|----------|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| DATABASE_URL             | yes      | --            | postgres://postgres:postgres@localhost/el_monitorro                                                                                                                                 |
| TELEGRAM_BOT_TOKEN       | yes      | --            | 6666618370:AAGx5YhNQvUG4eUcQXN-OB_a09ZzYl6aaaa                                                                                                                                      |
| DATABASE_POOL_SIZE       | no       | 5             | The maximum number of connections for global connection pool (global per binary except if ALL_BINARIES is set to true).                                                             |
| ALL_BINARIES             | no       | --            | If this var is set, all services will be started in the main binary                                                                                                                 |
| TELEGRAM_BOT_HANDLE      | no       | --            | This value is used during parsing of commands. If you set autocompletion menu for your bot,  the bot will understand commands like `/subscribe@handle` along with just `/subscribe` |
| SUBSCRIPTION_LIMIT       | no       | 20            |                                                                                                                                                                                     |
| SYNC_INTERVAL_SECONDS    | no       | 60            | The bot tries to sync feeds every `SYNC_INTERVAL_SECONDS` seconds                                                                                                                   |
| SYNC_WORKERS_NUMBER      | no       | 1             | The number of workers to sync feeds                                                                                                                                                 |
| DELIVER_INTERVAL_SECONDS | no       | 60            | The bot tries to deliver new feed items every `DELIVER_INTERVAL_SECONDS` seconds                                                                                                    |
| DELIVER_WORKERS_NUMBER   | no       | 1             | The number of workers to deliver updates                                                                                                                                            |
| CLEAN_INTERVAL_SECONDS   | no       | 3600          | The bot cleans old feed items and feeds without subscriptions every `CLEAN_INTERVAL_SECONDS` seconds                                                                                |
| CLEAN_WORKERS_NUMBER     | no       | 1             | The number of workers to remove old data                                                                                                                                            |
| OWNER_TELEGRAM_ID        | no       | --            | If this value is set, the bot will process commands only from the specified user id                                                                                                      |
| REQUEST_TIMEOUT          | no       | 5             | Timeout in seconds for feed syncing requests                                                                                                                                        |
| ADMIN_TELEGRAM_ID        | no       | --            | If this value is set, `/info` command with stats is available for ADMIN_TELEGRAM_ID                                                                                                 |

## Deployment suggestions

It's recommended to use a self hosted PostgreSQL instance but if it's not possible there are free services that can host it fo you:

- https://supabase.com  500mb db, up to 60 connections
- https://yugabyte.com, 10gb db, up to 10 connections
- https://bit.io, 3gb

## Using docker image

The image is published on docker hub under [ayratbadykov/el_monitorro](https://hub.docker.com/r/ayratbadykov/el_monitorro). It accepts additional env variables:

- `SETUP_DB` - if this variable is not empty, `diesel database setup` is run. It creates DB and runs migrations.
- `RUN_MIGRATION` - if this variable is not empty, `diesel migration run` is run. It just runs migrations.
- `BOT_BINARY` - depending on this variable, docker container will run one of four binaries. Possible values are `commands`, `sync`, `deliver`, `cleaner`. To run all services in the main binary, set:

```
RUN_MIGRATION=true
BOT_BINARY=commands
ALL_BINARIES=true
```

Run the docker container:

```sh
docker run --env-file ./.env --network host -t ayratbadykov/el_monitorro:latest
```

Notes:

- `--network host` is used so the docker container can access a host network if you're running Postgres on the same machine

You can check out an example of docker-compose file in the root directory of the project.

#### Creating a docker image from the latest master branch

Run the following command in the `el_monitorro` directory to build the image from the master branch:

```sh
docker build ./ -t ayratbadykov/el_monitorro:latest
```
