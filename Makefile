UI_DIST := ui/dist

.PHONY: default run clean

default: run

run: $(UI_DIST)
	cargo run

$(UI_DIST):
	@echo "Building UI..."
	cd ui && bun install && bun run build

clean:
	rm -rf $(UI_DIST)
