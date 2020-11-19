<p align="center"><img src="el_monitorro_logo.png" alt="el_monitorro" height="300px"></p>

[![paypal](https://www.paypalobjects.com/en_US/i/btn/btn_donateCC_LG.gif)](https://paypal.me/ayrat555)

# el_monitorro

`el_monitorro` is RSS, Atom and JSON feed reader as a Telegram bot.

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
- bot_space - defines a space character
- bot_new_line - defines a new line character
Example: /set_template https://www.badykov.com/feed.xml bot_datebot_spacebot_item_namebot_new_linebot_item_description.

/get_template url - get a template for the subscription

/set_global_template - set global template. This template will be used for all subscriptions. If the subscription has its own template, the subscription template will be used. See /set_template for available fields.

/get_global_template - get global template
```

### Common info

- Feed updates check interval is 1 minute.
- Unread items delivery interval is 1 minute.
- The number of subscriptions is limited to 20.

# Setup

## Automatic deploy to Heroku

1. Customize `.env` and `Procfile` with personal settings
2. Click to deploy this application on Heroku:

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy)

## Manually setup

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

3. Start a bot

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
