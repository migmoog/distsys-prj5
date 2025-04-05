all: comp1

comp%: prj5
	docker compose -f testcases/docker-compose-testcase-$*.yml up --remove-orphans

prj5:
	docker compose -f builder.yml build

.PHONY: teardown%
teardown%:
	docker compose -f testcases/docker-compose-testcase-$*.yml down
