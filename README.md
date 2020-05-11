<p align="center"><img src="el_monitorro_logo.png" alt="el_monitorro" height="300px"></p>

# el_monitorro

`el_monitorro` is RSS reader as a Telegram bot.

It's available at [@el_monitorro_bot](https://t.me/el_monitorro_bot).

# Usage

### Commands

```
/subscribe rss_url - subscribe to rss feed
/unsubscribe rss_url - unsubscribe from rss feed
/list_subscriptions - list your subscriptions
/help - show available commands
```

### Update interval

RSS Feeds updates check interval is 1 minute.
Unread items delivery interval is 1 minute.


# Setup

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

```
RUST_LOG=info cargo run
```
