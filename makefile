build:
	cargo build-sbf

deploy:
	solana program deploy target/deploy/dao_program.so