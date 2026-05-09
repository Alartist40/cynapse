.PHONY: build run dev clean tidy

BINARY := cynapse

build: tidy
	@echo "Building CYNAPSE..."
	@mkdir -p bin
	@go build -ldflags="-s -w" -o bin/$(BINARY) ./cmd/cynapse
	@echo "✓ bin/$(BINARY)"

run: build
	@./bin/$(BINARY)

dev:
	@go run ./cmd/cynapse

tidy:
	@go mod tidy

clean:
	@rm -rf bin/ data/

# Pi builds
build-pi:
	@GOOS=linux GOARCH=arm64 go build -ldflags="-s -w" -o bin/$(BINARY)-arm64 ./cmd/cynapse
	@echo "✓ bin/$(BINARY)-arm64 (Pi 5)"

build-pi-zero:
	@GOOS=linux GOARCH=arm GOARM=7 go build -ldflags="-s -w" -o bin/$(BINARY)-armv7 ./cmd/cynapse
	@echo "✓ bin/$(BINARY)-armv7 (Pi Zero 2W)"
