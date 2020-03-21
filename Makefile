
serve: build
	backend/target/debug/backend

build:
	cd backend && cargo build && cd ..

serve-rel: build-rel
	backend/target/release/backend

build-rel:
	cd backend && cargo build --release && cd ..


.PHONY: serve build serve-rel build-rel
