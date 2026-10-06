.DEFAULT_GOAL := help

CARGO ?= cargo
CLI_PACKAGE ?= sf-cli

SOURCE ?= tests/fixtures/parquet/sales.parquet
COLS ?= price,quantity,revenue
OUTLIER_COLS ?= revenue
METHOD ?= pearson
RECIPE ?= tests/fixtures/recipes/eda_full.toml
TABLE ?=
QUERY ?=
OUT ?=
FORMAT ?=

.PHONY: help build check fmt fmt-check clippy test clean dev \
	run-summary run-corr run-outlier run-recipe

help:
	@printf '%s\n' \
		'Usage: make <target> [VAR=value]' \
		'' \
		'Development targets:' \
		'  build        Build the full workspace' \
		'  check        Run cargo check for the workspace' \
		'  fmt          Format the workspace' \
		'  fmt-check    Verify formatting without changing files' \
		'  clippy       Run clippy for the workspace' \
		'  test         Run the workspace test suite' \
		'  dev          Run the default CLI flow against bundled fixtures' \
		'  clean        Remove build artifacts' \
		'' \
		'CLI targets:' \
		'  run-summary  Run EDA summary against SOURCE/COLS' \
		'  run-corr     Run EDA correlation against SOURCE/COLS/METHOD' \
		'  run-outlier  Run outlier detection against SOURCE/OUTLIER_COLS' \
		'  run-recipe   Run a recipe file from RECIPE' \
		'' \
		'Common vars:' \
		'  SOURCE=tests/fixtures/parquet/sales.parquet' \
		'  COLS=price,quantity,revenue' \
		'  OUTLIER_COLS=revenue' \
		'  METHOD=pearson' \
		'  RECIPE=tests/fixtures/recipes/eda_full.toml' \
		'  TABLE=orders      QUERY=select * from orders' \
		'  OUT=output/result.json  FORMAT=json'

build:
	$(CARGO) build --workspace

check:
	$(CARGO) check --workspace

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

clippy:
	$(CARGO) clippy --workspace --all-targets

test:
	$(CARGO) test --workspace

dev: run-summary

clean:
	$(CARGO) clean

run-summary:
	$(CARGO) run -p $(CLI_PACKAGE) -- eda summary --source "$(SOURCE)" $(if $(TABLE),--table "$(TABLE)",) $(if $(QUERY),--query "$(QUERY)",) --cols "$(COLS)" $(if $(OUT),--out "$(OUT)",) $(if $(FORMAT),--format "$(FORMAT)",)

run-corr:
	$(CARGO) run -p $(CLI_PACKAGE) -- eda corr --source "$(SOURCE)" $(if $(TABLE),--table "$(TABLE)",) $(if $(QUERY),--query "$(QUERY)",) --cols "$(COLS)" --method "$(METHOD)" $(if $(OUT),--out "$(OUT)",) $(if $(FORMAT),--format "$(FORMAT)",)

run-outlier:
	$(CARGO) run -p $(CLI_PACKAGE) -- stats outlier --source "$(SOURCE)" $(if $(TABLE),--table "$(TABLE)",) $(if $(QUERY),--query "$(QUERY)",) --cols "$(OUTLIER_COLS)" $(if $(OUT),--out "$(OUT)",) $(if $(FORMAT),--format "$(FORMAT)",)

run-recipe:
	$(CARGO) run -p $(CLI_PACKAGE) -- run --recipe "$(RECIPE)"
