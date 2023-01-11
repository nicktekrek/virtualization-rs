.PHONY: release

run:
	cargo build --example vm-cli
	codesign -f --entitlement virtualization_rs.entitlements -s - target/debug/examples/vm-cli
	./target/debug/examples/vm-cli

debug:
	cargo build --example simplevm
	codesign -f --entitlement virtualization_rs.entitlements -s - target/debug/examples/simplevm

release:
	cargo build --release --example simplevm
	codesign -f --entitlement virtualization_rs.entitlements -s - target/release/examples/simplevm

check:
	cargo check

clean:
	cargo clean
