### Following is not 100% accurate or tested
## Uncomment first line if first deploy, uncomment second for the following
#release: ./target/release/diesel database setup
release: ./target/release/diesel migration run

### Uncomment this to use with Heroku paid tier
#worker: ./target/release/el_monitorro
#cleaner: ./target/release/cleaner
#syncer: ./target/release/sync
#deliver: ./target/release/deliver

### Uncomment this to use with Heroku free tier (run 2 max at the same time)
worker: ./target/release/el_monitorro & ./target/release/deliver & ./target/release/sync
cleaner: ./target/release/cleaner
