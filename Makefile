db:
	docker run --rm -d --name postgres -p 5432:5432 \
  -e POSTGRES_DB=el_monitorro \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  postgres:13.4

clippy:
	cargo clippy --all-features

diesel:
	diesel migration run

stop:
	docker kill postgres

test:
	cargo test -- --color always --nocapture

ignored:
	cargo test -- --color always --nocapture --ignored

doc:
	cargo doc --open
