.PHONY: vet

vet:
	go vet ./...
	go mod tidy
