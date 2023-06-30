include .env
export $(shell sed 's/=.*//' .env)

export COMPOSE_DOCKER_CLI_BUILD=1
export DOCKER_BUILDKIT=1
EXPORT = export RUSTPATH=$(PWD)



migration:
	$(EXPORT) && sqlx migrate add -r ${title}

upgrade:
	$(EXPORT) && sqlx migrate run --database-url $(DATABASE_URL)

downgrade:
	$(EXPORT) && sqlx migrate revert --database-url $(DATABASE_URL)

test:
	$(EXPORT) && cargo test -- --test-threads 1 --nocapture                        

checks:
	$(EXPORT) && cargo fmt
	$(EXPORT) && cargo clippy

delete-git-branch-except-main:
	git branch | grep -v "main" | xargs git branch -D